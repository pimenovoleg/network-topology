use std::net::IpAddr;

use crate::server::{
    services::{
        definitions::ServiceDefinitionRegistry,
        r#impl::{
            base::{
                DiscoverySessionServiceMatchParams, ServiceMatchBaselineParams,
                ServiceMatchServiceParams,
            },
            virtualization::ServiceVirtualization,
        },
    },
    shared::types::metadata::TypeMetadataProvider,
    subnets::r#impl::types::SubnetType,
};
use anyhow::{Error, anyhow};
use mac_oui::Oui;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumDiscriminants, IntoStaticStr};

use crate::server::{
    hosts::r#impl::ports::{Port, PortBase},
    services::r#impl::endpoints::Endpoint,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct MatchResult {
    pub ports: Vec<Port>,
    pub endpoint: Option<Endpoint>,
    pub mac_vendor: Option<String>,
    pub details: MatchDetails,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchDetails {
    pub reason: MatchReason,
    pub confidence: MatchConfidence,
}

impl MatchDetails {
    pub fn new_certain(reason_str: &str) -> Self {
        Self {
            reason: MatchReason::Reason(reason_str.to_string()),
            confidence: MatchConfidence::Certain,
        }
    }

    pub fn reason_string(&self) -> String {
        match &self.reason {
            MatchReason::Container(string, _) => string.clone(),
            MatchReason::Reason(string) => string.clone(),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Display, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "lowercase")]
pub enum MatchReason {
    Reason(String),
    #[serde(rename = "container")]
    Container(String, Vec<MatchReason>),
}

#[derive(Debug, Clone, Hash, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MatchConfidence {
    NotApplicable = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    Certain = 4,
}

impl MatchConfidence {
    pub fn as_str(&self) -> &'static str {
        match self {
            MatchConfidence::NotApplicable => "Not Applicable",
            MatchConfidence::Low => "Low",
            MatchConfidence::Medium => "Medium",
            MatchConfidence::High => "High",
            MatchConfidence::Certain => "Certain",
        }
    }
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(IntoStaticStr))]
pub enum Pattern<'a> {
    /// Match any of the listed patterns
    AnyOf(Vec<Pattern<'a>>),

    /// Must match all of the listed patterns
    AllOf(Vec<Pattern<'a>>),

    /// Inverse of pattern
    Not(Box<Pattern<'a>>),

    /// Whether or not a specific port is open on the host
    Port(PortBase),

    /// Whether or not an endpoint provided a specific response
    /// PortBase
    /// path: &str - ie "/", "/admin", etc
    /// expected response: &str - String to match on in response
    Endpoint(PortBase, &'a str, &'a str),

    /// Whether the subnet that the host was found on matches a subnet type
    SubnetIsType(SubnetType),

    /// Whether the host IP is found in the daemon's routing table. WARNING: Using this will automatically classify the service as a Layer3 service, and the service will only be able to bind to interfaces (ports and port bindings will be ignored)
    IsGateway,

    /// Whether the vendor derived from the mac address (https://gist.github.com/aallan/b4bb86db86079509e6159810ae9bd3e4) matches the provided str
    MacVendor(&'static str),

    /// Custom evaluation of discovery match params
    /// fn - constraint function
    /// &'a str - match reason (describe what it means if function evaluates true)
    /// &'a str - no match reason (describe what it means if function evaluates false)
    /// MatchConfdence - confidence level that match uniquely identifies service
    Custom(
        fn(&DiscoverySessionServiceMatchParams) -> bool,
        &'a str,
        &'a str,
        MatchConfidence,
    ),

    /// Whether the host is a docker container
    DockerContainer,

    /// No match pattern (only added manually or by the system)
    None,
}

// https://gist.github.com/aallan/b4bb86db86079509e6159810ae9bd3e4
pub struct Vendor;
impl Vendor {
    pub const PHILIPS: &'static str = "Philips Lighting BV";
    pub const HP: &'static str = "HP Inc.";
    pub const EERO: &'static str = "eero Inc";
    pub const TPLINK: &'static str = "TP-LINK TECHNOLOGIES CO.,LTD";
    pub const UBIQUITI: &'static str = "Ubiquiti Networks Inc";
    pub const GOOGLE: &'static str = "Google, Inc.";
    pub const NEST: &'static str = "Nest Labs Inc.";
    pub const AMAZON: &'static str = "Amazon Technologies Inc.";
    pub const SONOS: &'static str = "Sonos, Inc.";
    pub const ECOBEE: &'static str = "ecobee inc";
    pub const ROKU: &'static str = "Roku, Inc";
}

impl Pattern<'_> {
    pub fn matches(
        &self,
        params: &DiscoverySessionServiceMatchParams,
    ) -> Result<MatchResult, Error> {
        // Return ports + endpoint that matched, if any

        let DiscoverySessionServiceMatchParams {
            gateway_ips,
            baseline_params,
            service_params,
            daemon_id,
            ..
        } = params;

        let ServiceMatchBaselineParams {
            subnet,
            interface,
            endpoint_responses,
            virtualization,
            ..
        } = baseline_params;

        let ServiceMatchServiceParams {
            unbound_ports,
            service_definition,
            ..
        } = service_params;

        match self {
            Pattern::Port(port_base) => {
                if let Some(matched_port) = unbound_ports.iter().find(|p| **p == *port_base) {
                    let mut all_other_services_ports: Vec<PortBase> =
                        ServiceDefinitionRegistry::all_service_definitions()
                            .iter()
                            .filter(|s| s.id() != service_definition.id())
                            .flat_map(|s| s.discovery_pattern().ports())
                            .collect();

                    all_other_services_ports.sort_by_key(|p| (p.number(), p.protocol()));
                    all_other_services_ports.dedup();

                    let is_unique_to_service =
                        port_base.is_custom() && !all_other_services_ports.contains(port_base);

                    let (reason, confidence) = if port_base.is_custom() && is_unique_to_service {
                        (
                            format!(
                                "Port {} is open and is not used in other service match patterns",
                                port_base,
                            ),
                            MatchConfidence::Medium,
                        )
                    } else {
                        (
                            format!(
                                "Port {} is open but is used in other service match patterns",
                                port_base
                            ),
                            MatchConfidence::Low,
                        )
                    };

                    Ok(MatchResult {
                        ports: vec![Port::new(*matched_port)],
                        endpoint: None,
                        mac_vendor: None,
                        details: MatchDetails {
                            reason: MatchReason::Reason(reason),
                            confidence,
                        },
                    })
                } else {
                    Err(anyhow!("Port {} is not open", port_base))
                }
            }

            Pattern::Endpoint(port_base, path, expected_response) => {
                let endpoint = Endpoint::for_pattern(*port_base, path);

                if let Some(actual) = endpoint_responses.iter().find(|actual| {
                    // Compare without IP since pattern endpoints don't have IPs
                    actual.endpoint.protocol == endpoint.protocol
                        && actual.endpoint.port_base.number() == endpoint.port_base.number()
                        && actual.endpoint.path == endpoint.path
                        && actual
                            .response
                            .to_lowercase()
                            .contains(&expected_response.to_lowercase())
                }) {
                    Ok(MatchResult {
                        ports: vec![Port::new(actual.endpoint.port_base)],
                        endpoint: Some(actual.endpoint.clone()),
                        mac_vendor: None,
                        details: MatchDetails {
                            reason: MatchReason::Reason(format!(
                                "Response from {} contained \"{}\"",
                                actual.endpoint, expected_response
                            )),
                            confidence: MatchConfidence::High,
                        },
                    })
                } else {
                    Err(anyhow!(
                        "Response from {} did not contain \"{}\"",
                        endpoint,
                        expected_response
                    ))
                }
            }

            Pattern::MacVendor(vendor_string) => {
                if let Some(mac) = interface.base.mac_address {
                    let Ok(oui_db) = Oui::default() else {
                        return Err(anyhow!("Could not load Oui database"));
                    };
                    let Ok(Some(entry)) = Oui::lookup_by_mac(&oui_db, &mac.to_string()) else {
                        return Err(anyhow!(
                            "Could find vendor for mac address {} in Oui database",
                            mac
                        ));
                    };

                    let normalize = |s: &str| -> String {
                        s.trim()
                            .to_lowercase()
                            .chars()
                            .filter(|c| c.is_alphanumeric())
                            .collect()
                    };

                    let vendor_string = normalize(vendor_string);
                    let entry_string = normalize(&entry.company_name);

                    if vendor_string == entry_string {
                        Ok(MatchResult {
                            ports: vec![],
                            endpoint: None,
                            mac_vendor: Some(entry.company_name.clone()),
                            details: MatchDetails {
                                reason: MatchReason::Reason(format!(
                                    "Mac address is from vendor {}",
                                    entry.company_name
                                )),
                                confidence: MatchConfidence::Medium,
                            },
                        })
                    } else {
                        Err(anyhow!("Mac address is not from vendor {}", vendor_string))
                    }
                } else {
                    Err(anyhow!(
                        "Interface {} does not have a mac address",
                        interface.base.ip_address
                    ))
                }
            }

            Pattern::Not(pattern) => match pattern.matches(params) {
                Ok(result) => Err(anyhow!("{}", result.details.reason)),
                Err(e) => Ok(MatchResult {
                    ports: vec![],
                    endpoint: None,
                    mac_vendor: None,
                    details: MatchDetails {
                        reason: MatchReason::Reason(format!("{}", e)),
                        confidence: MatchConfidence::Low,
                    },
                }),
            },

            Pattern::AnyOf(patterns) => {
                let mut ports = Vec::new();
                let mut endpoint = None;
                let mut mac_vendor = None;
                let mut any_matched = false;
                let mut confidence = MatchConfidence::Low;
                let mut reasons = Vec::new();
                let mut no_match_errors = String::new();
                patterns.iter().for_each(|p| match p.matches(params) {
                    Ok(result) => {
                        any_matched = true;
                        ports.extend(result.ports);
                        reasons.push(result.details.reason);

                        if result.endpoint.is_some() && endpoint.is_none() {
                            endpoint = result.endpoint;
                        }

                        if result.mac_vendor.is_some() && mac_vendor.is_none() {
                            mac_vendor = result.mac_vendor;
                        }

                        if result.details.confidence > confidence {
                            confidence = result.details.confidence;
                        }
                    }
                    Err(e) => {
                        no_match_errors = no_match_errors.clone() + ", " + &e.to_string();
                    }
                });

                if any_matched {
                    Ok(MatchResult {
                        ports,
                        endpoint: None,
                        mac_vendor: None,
                        details: MatchDetails {
                            reason: MatchReason::Container("Any of".to_string(), reasons),
                            confidence,
                        },
                    })
                } else {
                    Err(anyhow!(no_match_errors))
                }
            }

            Pattern::AllOf(patterns) => {
                let mut all_matched = true;
                let mut ports = Vec::new();
                let mut endpoint = None;
                let mut mac_vendor = None;
                let mut matched_confidences = Vec::new();
                let mut reasons = Vec::new();
                let mut no_match_errors = String::new();
                patterns.iter().for_each(|p| match p.matches(params) {
                    Ok(result) => {
                        ports.extend(result.ports);
                        reasons.push(result.details.reason);
                        matched_confidences.push(result.details.confidence);

                        if result.endpoint.is_some() && endpoint.is_none() {
                            endpoint = result.endpoint;
                        }

                        if result.mac_vendor.is_some() && mac_vendor.is_none() {
                            mac_vendor = result.mac_vendor;
                        }
                    }
                    Err(e) => {
                        all_matched = false;
                        no_match_errors = no_match_errors.clone() + ", " + &e.to_string();
                    }
                });

                if all_matched {
                    matched_confidences.sort();

                    let max_confidence =
                        matched_confidences.last().unwrap_or(&MatchConfidence::Low);

                    // Boost confidence if multiple lower-confidence patterns are matched
                    let confidence = if matches!(
                        max_confidence,
                        MatchConfidence::Low | MatchConfidence::Medium
                    ) && matched_confidences.len() > 3
                    {
                        match max_confidence {
                            MatchConfidence::Low => MatchConfidence::Medium,
                            MatchConfidence::Medium => MatchConfidence::High,
                            _ => *max_confidence,
                        }
                    } else {
                        *max_confidence
                    };

                    Ok(MatchResult {
                        ports,
                        endpoint: None,
                        mac_vendor: None,
                        details: MatchDetails {
                            reason: MatchReason::Container("All of".to_string(), reasons),
                            confidence,
                        },
                    })
                } else {
                    Err(anyhow!(no_match_errors))
                }
            }

            Pattern::IsGateway => {
                let gateway_ips_in_subnet: Vec<_> = gateway_ips
                    .iter()
                    .filter(|g| subnet.base.cidr.contains(g))
                    .collect();

                let count_gateways_in_subnet = gateway_ips_in_subnet.len();
                let host_ip_in_routing_table =
                    gateway_ips_in_subnet.contains(&&interface.base.ip_address);

                let last_octet_1_or_254 = match interface.base.ip_address {
                    IpAddr::V4(ipv4) => {
                        let octets = ipv4.octets();
                        octets[3] == 1 || octets[3] == 254
                    }
                    IpAddr::V6(ipv6) => {
                        let segments = ipv6.segments();
                        segments[7] == 1 || segments[7] == 254
                    }
                };

                let mut reason = String::new();

                let is_gateway = if host_ip_in_routing_table {
                    reason = format!(
                        "Host IP address is in routing table of daemon {}",
                        daemon_id
                    );
                    true
                } else if last_octet_1_or_254 && count_gateways_in_subnet == 0 {
                    // Likely a gateway if common IP and no other gateways found
                    reason = format!(
                        "No other gateways in subnet {} and IP address ends in 1 or 254",
                        subnet.base.cidr
                    );
                    true
                } else {
                    false
                };

                if is_gateway {
                    Ok(MatchResult {
                        ports: vec![],
                        endpoint: None,
                        mac_vendor: None,
                        details: MatchDetails {
                            reason: MatchReason::Reason(reason),
                            confidence: MatchConfidence::High,
                        },
                    })
                } else {
                    Err(anyhow!(
                        "IP address is not in routing table, and does not end in 1 or 254 with no other gateways identified in subnet"
                    ))
                }
            }

            Pattern::SubnetIsType(subnet_type) => {
                if &subnet.base.subnet_type == subnet_type {
                    Ok(MatchResult {
                        ports: vec![],
                        endpoint: None,
                        mac_vendor: None,
                        details: MatchDetails {
                            reason: MatchReason::Reason(format!(
                                "Subnet {} is type {}",
                                subnet.base.cidr,
                                subnet_type.name()
                            )),
                            confidence: MatchConfidence::Low,
                        },
                    })
                } else {
                    Err(anyhow!(
                        "Subnet {} is not type {}",
                        subnet.base.cidr,
                        subnet_type.name()
                    ))
                }
            }

            Pattern::Custom(constraint_function, reason, no_match_reason, confidence) => {
                if constraint_function(params) {
                    Ok(MatchResult {
                        ports: vec![],
                        endpoint: None,
                        mac_vendor: None,
                        details: MatchDetails {
                            reason: MatchReason::Reason(reason.to_string()),
                            confidence: *confidence,
                        },
                    })
                } else {
                    let no_match_reason = no_match_reason.to_string();
                    Err(anyhow!(no_match_reason))
                }
            }

            Pattern::DockerContainer => match virtualization {
                Some(ServiceVirtualization::Docker(..)) => Ok(MatchResult {
                    ports: vec![],
                    endpoint: None,
                    mac_vendor: None,
                    details: MatchDetails {
                        reason: MatchReason::Reason(
                            "Service is running in docker container".to_string(),
                        ),
                        confidence: MatchConfidence::Low,
                    },
                }),
                _ => Err(anyhow!("Service is not running in a docker container")),
            },

            Pattern::None => Err(anyhow!("No match pattern provided")),
        }
    }

    /// Get all ports which need to be scanned for a given service's match pattern
    pub fn ports(&self) -> Vec<PortBase> {
        match self {
            Pattern::Port(port) => vec![*port],
            Pattern::AnyOf(patterns) | Pattern::AllOf(patterns) => {
                patterns.iter().flat_map(|p| p.ports().to_vec()).collect()
            }
            _ => vec![],
        }
    }

    /// Get all endpoints which need to be scanned for a given service's match pattern
    pub fn endpoints(&self) -> Vec<Endpoint> {
        match self {
            Pattern::Endpoint(port_base, path, _) => vec![Endpoint::for_pattern(*port_base, path)],
            Pattern::AnyOf(patterns) | Pattern::AllOf(patterns) => patterns
                .iter()
                .flat_map(|p| p.endpoints().to_vec())
                .collect(),
            _ => vec![],
        }
    }

    /// Whether service uses IsGateway as a positive match signal -> service is_gateway = trues
    pub fn contains_gateway_ip_pattern(&self) -> bool {
        match self {
            Pattern::IsGateway => true,
            Pattern::AllOf(patterns) | Pattern::AnyOf(patterns) => {
                patterns.iter().any(|p| p.contains_gateway_ip_pattern())
            }
            _ => false,
        }
    }
}
