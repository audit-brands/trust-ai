//! Health checking system for local AI providers

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Context as _;
use tokio::sync::RwLock;

use tracing::{debug, error, info, warn};

use crate::config::local_ai::{LocalAiConfig, ProviderHealthChecker, ProviderHealthStatus};

/// Health monitoring service for local AI providers
pub struct HealthMonitor {
    config: LocalAiConfig,
    health_status: Arc<RwLock<HashMap<String, ProviderHealthInfo>>>,
    checkers: HashMap<String, Box<dyn ProviderHealthChecker>>,
}

/// Health information for a provider
#[derive(Debug, Clone)]
pub struct ProviderHealthInfo {
    /// Current health status
    pub status: ProviderHealthStatus,
    /// Last check timestamp
    pub last_checked: Instant,
    /// Consecutive failure count
    pub consecutive_failures: u32,
    /// Consecutive success count
    pub consecutive_successes: u32,
    /// Average response time over last 10 checks
    pub avg_response_time: Duration,
    /// Check history (last 10 results)
    pub check_history: Vec<HealthCheckResult>,
}

/// Result of a health check
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// Timestamp of the check
    pub timestamp: Instant,
    /// Whether the check was successful
    pub success: bool,
    /// Response time
    pub response_time: Duration,
    /// Error message if failed
    pub error: Option<String>,
}

/// Health monitor events
#[derive(Debug, Clone)]
pub enum HealthEvent {
    /// Provider became healthy
    ProviderHealthy {
        provider_name: String,
        response_time: Duration,
    },
    /// Provider became unhealthy
    ProviderUnhealthy {
        provider_name: String,
        reason: String,
        consecutive_failures: u32,
    },
    /// Provider status degraded
    ProviderDegraded {
        provider_name: String,
        reason: String,
        response_time: Duration,
    },
    /// Provider recovered from degraded state
    ProviderRecovered {
        provider_name: String,
        response_time: Duration,
    },
}

impl HealthMonitor {
    /// Create a new health monitor
    pub async fn new(config: LocalAiConfig) -> anyhow::Result<Self> {
        let mut checkers = HashMap::new();
        
        // Create health checkers for enabled providers
        for (name, provider_config) in config.enabled_providers() {
            if let Ok(checker) = provider_config.create_health_checker() {
                checkers.insert(name.clone(), checker);
            } else {
                warn!("Failed to create health checker for provider: {}", name);
            }
        }

        Ok(Self {
            config,
            health_status: Arc::new(RwLock::new(HashMap::new())),
            checkers,
        })
    }

    /// Start the health monitoring service
    pub async fn start(&self) -> anyhow::Result<()> {
        info!("Starting health monitor for {} providers", self.checkers.len());

        // Perform initial health checks
        self.perform_initial_checks().await?;

        // Start periodic health checks for each provider
        for (provider_name, _) in &self.checkers {
            self.start_provider_monitoring(provider_name.clone()).await;
        }

        Ok(())
    }

    /// Perform initial health checks for all providers
    async fn perform_initial_checks(&self) -> anyhow::Result<()> {
        info!("Performing initial health checks");

        for (provider_name, checker) in &self.checkers {
            match self.check_provider_health(provider_name, checker).await {
                Ok(info) => {
                    let mut status = self.health_status.write().await;
                    status.insert(provider_name.clone(), info);
                    info!("Initial health check completed for {}: {:?}", provider_name, status.get(provider_name).unwrap().status);
                }
                Err(e) => {
                    error!("Initial health check failed for {}: {}", provider_name, e);
                    // Insert unhealthy status
                    let unhealthy_info = ProviderHealthInfo {
                        status: ProviderHealthStatus::Unhealthy {
                            reason: format!("Initial check failed: {}", e),
                            response_time: Duration::from_millis(0),
                        },
                        last_checked: Instant::now(),
                        consecutive_failures: 1,
                        consecutive_successes: 0,
                        avg_response_time: Duration::from_millis(0),
                        check_history: vec![],
                    };
                    let mut status = self.health_status.write().await;
                    status.insert(provider_name.clone(), unhealthy_info);
                }
            }
        }

        Ok(())
    }

    /// Start monitoring for a specific provider
    async fn start_provider_monitoring(&self, provider_name: String) {
        let provider_config = match self.config.providers.get(&provider_name) {
            Some(config) => config,
            None => {
                error!("Provider configuration not found: {}", provider_name);
                return;
            }
        };

        let interval_duration = provider_config.health_check.interval_duration();
        let _health_status = Arc::clone(&self.health_status);
        let _checker = match self.checkers.get(&provider_name) {
            Some(checker) => checker,
            None => {
                error!("Health checker not found for provider: {}", provider_name);
                return;
            }
        };

        let provider_name_clone = provider_name.clone();
        
        // Note: In a real implementation, we would spawn this as a background task
        // For now, we'll just log that monitoring would start
        info!(
            "Would start health monitoring for {} with interval {:?}",
            provider_name_clone, interval_duration
        );
    }

    /// Check health of a specific provider
    async fn check_provider_health(
        &self,
        provider_name: &str,
        checker: &Box<dyn ProviderHealthChecker>,
    ) -> anyhow::Result<ProviderHealthInfo> {
        let start_time = Instant::now();
        
        debug!("Checking health for provider: {}", provider_name);
        
        match checker.check_health().await {
            Ok(status) => {
                let response_time = start_time.elapsed();
                let check_result = HealthCheckResult {
                    timestamp: start_time,
                    success: status.is_usable(),
                    response_time,
                    error: None,
                };

                // Get current info or create new
                let current_info = {
                    let health_status = self.health_status.read().await;
                    health_status.get(provider_name).cloned()
                };

                let info = self.update_health_info(current_info, status, check_result);
                
                debug!(
                    "Health check completed for {}: {:?} ({}ms)",
                    provider_name,
                    info.status,
                    response_time.as_millis()
                );

                Ok(info)
            }
            Err(e) => {
                let response_time = start_time.elapsed();
                let error_msg = format!("Health check failed: {}", e);
                
                let check_result = HealthCheckResult {
                    timestamp: start_time,
                    success: false,
                    response_time,
                    error: Some(error_msg.clone()),
                };

                let unhealthy_status = ProviderHealthStatus::Unhealthy {
                    reason: error_msg,
                    response_time,
                };

                let current_info = {
                    let health_status = self.health_status.read().await;
                    health_status.get(provider_name).cloned()
                };

                let info = self.update_health_info(current_info, unhealthy_status, check_result);
                
                warn!(
                    "Health check failed for {}: {} ({}ms)",
                    provider_name,
                    e,
                    response_time.as_millis()
                );

                Ok(info)
            }
        }
    }

    /// Update health information with new check result
    fn update_health_info(
        &self,
        current_info: Option<ProviderHealthInfo>,
        new_status: ProviderHealthStatus,
        check_result: HealthCheckResult,
    ) -> ProviderHealthInfo {
        let now = Instant::now();
        
        match current_info {
            Some(mut info) => {
                // Update status
                info.status = new_status.clone();
                info.last_checked = now;
                
                // Update failure/success counters
                if check_result.success {
                    info.consecutive_successes += 1;
                    info.consecutive_failures = 0;
                } else {
                    info.consecutive_failures += 1;
                    info.consecutive_successes = 0;
                }
                
                // Update check history (keep last 10)
                info.check_history.push(check_result);
                if info.check_history.len() > 10 {
                    info.check_history.remove(0);
                }
                
                // Update average response time
                let total_time: Duration = info.check_history.iter()
                    .map(|result| result.response_time)
                    .sum();
                info.avg_response_time = total_time / info.check_history.len() as u32;
                
                info
            }
            None => {
                // Create new info
                ProviderHealthInfo {
                    status: new_status,
                    last_checked: now,
                    consecutive_failures: if check_result.success { 0 } else { 1 },
                    consecutive_successes: if check_result.success { 1 } else { 0 },
                    avg_response_time: check_result.response_time,
                    check_history: vec![check_result],
                }
            }
        }
    }

    /// Get current health status for all providers
    pub async fn get_health_status(&self) -> HashMap<String, ProviderHealthStatus> {
        let health_status = self.health_status.read().await;
        health_status
            .iter()
            .map(|(name, info)| (name.clone(), info.status.clone()))
            .collect()
    }

    /// Get detailed health information for all providers
    pub async fn get_detailed_health_info(&self) -> HashMap<String, ProviderHealthInfo> {
        let health_status = self.health_status.read().await;
        health_status.clone()
    }

    /// Get health status for a specific provider
    pub async fn get_provider_health(&self, provider_name: &str) -> Option<ProviderHealthStatus> {
        let health_status = self.health_status.read().await;
        health_status.get(provider_name).map(|info| info.status.clone())
    }

    /// Check if a provider is healthy
    pub async fn is_provider_healthy(&self, provider_name: &str) -> bool {
        if let Some(status) = self.get_provider_health(provider_name).await {
            matches!(status, ProviderHealthStatus::Healthy { .. })
        } else {
            false
        }
    }

    /// Check if a provider is usable (healthy or degraded)
    pub async fn is_provider_usable(&self, provider_name: &str) -> bool {
        if let Some(status) = self.get_provider_health(provider_name).await {
            status.is_usable()
        } else {
            false
        }
    }

    /// Force a health check for a specific provider
    pub async fn force_check(&self, provider_name: &str) -> anyhow::Result<ProviderHealthStatus> {
        let checker = self.checkers.get(provider_name)
            .with_context(|| format!("No health checker found for provider: {}", provider_name))?;

        let info = self.check_provider_health(provider_name, checker).await?;
        
        // Update stored status
        {
            let mut health_status = self.health_status.write().await;
            health_status.insert(provider_name.to_string(), info.clone());
        }

        Ok(info.status)
    }

    /// Force health checks for all providers
    pub async fn force_check_all(&self) -> anyhow::Result<HashMap<String, ProviderHealthStatus>> {
        let mut results = HashMap::new();

        for provider_name in self.checkers.keys() {
            match self.force_check(provider_name).await {
                Ok(status) => {
                    results.insert(provider_name.clone(), status);
                }
                Err(e) => {
                    error!("Failed to check health for {}: {}", provider_name, e);
                    results.insert(
                        provider_name.clone(),
                        ProviderHealthStatus::Unhealthy {
                            reason: format!("Check failed: {}", e),
                            response_time: Duration::from_millis(0),
                        },
                    );
                }
            }
        }

        Ok(results)
    }

    /// Get providers sorted by health (healthy first, then degraded, then unhealthy)
    pub async fn get_providers_by_health(&self) -> Vec<(String, ProviderHealthStatus)> {
        let health_status = self.health_status.read().await;
        let mut providers: Vec<_> = health_status
            .iter()
            .map(|(name, info)| (name.clone(), info.status.clone()))
            .collect();

        // Sort by health status priority
        providers.sort_by(|(_, a), (_, b)| {
            let priority_a = match a {
                ProviderHealthStatus::Healthy { .. } => 0,
                ProviderHealthStatus::Degraded { .. } => 1,
                ProviderHealthStatus::Unhealthy { .. } => 2,
            };
            let priority_b = match b {
                ProviderHealthStatus::Healthy { .. } => 0,
                ProviderHealthStatus::Degraded { .. } => 1,
                ProviderHealthStatus::Unhealthy { .. } => 2,
            };
            priority_a.cmp(&priority_b)
        });

        providers
    }
}

impl ProviderHealthInfo {
    /// Check if the provider has been consistently failing
    pub fn is_consistently_failing(&self, threshold: u32) -> bool {
        self.consecutive_failures >= threshold
    }

    /// Check if the provider has been consistently healthy
    pub fn is_consistently_healthy(&self, threshold: u32) -> bool {
        self.consecutive_successes >= threshold
    }

    /// Get the success rate over the check history
    pub fn success_rate(&self) -> f64 {
        if self.check_history.is_empty() {
            return 0.0;
        }

        let successful_checks = self.check_history.iter()
            .filter(|result| result.success)
            .count();

        successful_checks as f64 / self.check_history.len() as f64
    }

    /// Check if the provider is performing well
    pub fn is_performing_well(&self, max_response_time: Duration, min_success_rate: f64) -> bool {
        self.avg_response_time <= max_response_time && self.success_rate() >= min_success_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::local_ai::LocalAiConfig;
    use pretty_assertions::assert_eq;
    use std::time::Duration;

    #[tokio::test]
    async fn test_health_monitor_creation() {
        let config = LocalAiConfig::with_default_ollama();
        let actual = HealthMonitor::new(config).await;
        assert!(actual.is_ok());
    }

    #[tokio::test]
    async fn test_health_monitor_empty_config() {
        let config = LocalAiConfig::new();
        let monitor = HealthMonitor::new(config).await.unwrap();
        let health_status = monitor.get_health_status().await;
        assert_eq!(health_status.len(), 0);
    }

    #[test]
    fn test_provider_health_info_success_rate() {
        let mut fixture = ProviderHealthInfo {
            status: ProviderHealthStatus::Healthy {
                response_time: Duration::from_millis(100),
                models_available: 5,
                additional_info: None,
            },
            last_checked: Instant::now(),
            consecutive_failures: 0,
            consecutive_successes: 3,
            avg_response_time: Duration::from_millis(100),
            check_history: vec![
                HealthCheckResult {
                    timestamp: Instant::now(),
                    success: true,
                    response_time: Duration::from_millis(100),
                    error: None,
                },
                HealthCheckResult {
                    timestamp: Instant::now(),
                    success: false,
                    response_time: Duration::from_millis(200),
                    error: Some("Test error".to_string()),
                },
                HealthCheckResult {
                    timestamp: Instant::now(),
                    success: true,
                    response_time: Duration::from_millis(150),
                    error: None,
                },
            ],
        };

        let actual = fixture.success_rate();
        let expected = 2.0 / 3.0; // 2 successful out of 3 total
        assert!((actual - expected).abs() < 0.001);
    }

    #[test]
    fn test_provider_health_info_consistency_checks() {
        let fixture = ProviderHealthInfo {
            status: ProviderHealthStatus::Unhealthy {
                reason: "Test".to_string(),
                response_time: Duration::from_millis(0),
            },
            last_checked: Instant::now(),
            consecutive_failures: 5,
            consecutive_successes: 0,
            avg_response_time: Duration::from_millis(0),
            check_history: vec![],
        };

        assert!(fixture.is_consistently_failing(3));
        assert!(!fixture.is_consistently_failing(10));
        assert!(!fixture.is_consistently_healthy(1));
    }

    #[test]
    fn test_provider_health_info_performance() {
        let fixture = ProviderHealthInfo {
            status: ProviderHealthStatus::Healthy {
                response_time: Duration::from_millis(100),
                models_available: 5,
                additional_info: None,
            },
            last_checked: Instant::now(),
            consecutive_failures: 0,
            consecutive_successes: 5,
            avg_response_time: Duration::from_millis(200),
            check_history: vec![
                HealthCheckResult {
                    timestamp: Instant::now(),
                    success: true,
                    response_time: Duration::from_millis(200),
                    error: None,
                },
            ],
        };

        // Should perform well with lenient thresholds
        assert!(fixture.is_performing_well(Duration::from_millis(300), 0.8));
        
        // Should not perform well with strict thresholds
        assert!(!fixture.is_performing_well(Duration::from_millis(100), 0.8));
    }
}