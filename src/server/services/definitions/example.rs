use crate::server::services::definitions::{create_service, ServiceDefinitionFactory};
use crate::server::services::types::patterns::Pattern;
use crate::server::hosts::types::ports::PortBase;
use crate::server::services::types::types::ServiceDefinition;
use crate::server::services::types::categories::ServiceCategory;

#[derive(Default, Clone, Eq, PartialEq, Hash)]
pub struct YourService;

impl ServiceDefinition for YourService {
    fn name(&self) -> &'static str { "Your Service Name" } // < 15 chars
    fn description(&self) -> &'static str { "Brief description" } // < 60 chars
    fn category(&self) -> ServiceCategory { ServiceCategory::Web }
    
    fn discovery_pattern(&self) -> Pattern {
        // Choose appropriate pattern
        Pattern::Port(PortBase::new_tcp(8080))
    }
    
    // Optional overrides:
    // fn is_generic(&self) -> bool { false }
    // fn is_gateway(&self) -> bool { false }
    // fn icon(&self) -> &'static str { "icon-name" }
}

inventory::submit!(ServiceDefinitionFactory::new(create_service::<YourService>));