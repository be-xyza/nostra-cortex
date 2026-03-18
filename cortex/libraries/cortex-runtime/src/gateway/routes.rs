use crate::gateway::types::{
    GatewayIdempotencySemantics, GatewayRouteMetadata, GatewayTransactionBoundary,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayRouteDescriptor {
    pub method: String,
    pub path_template: String,
    pub idempotency_semantics: GatewayIdempotencySemantics,
    pub transaction_boundary: GatewayTransactionBoundary,
    pub expected_event_emissions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatewayRouteResolutionError {
    AmbiguousTemplateMatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayRouteMatch {
    pub metadata: GatewayRouteMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MatchCandidate {
    path_template: String,
    path_params: BTreeMap<String, String>,
    static_segments: usize,
    segment_count: usize,
}

pub fn resolve_route(
    descriptors: &[GatewayRouteDescriptor],
    method: &str,
    path: &str,
) -> Result<Option<GatewayRouteMatch>, GatewayRouteResolutionError> {
    let method = method.to_ascii_uppercase();
    let mut candidates = Vec::new();
    for descriptor in descriptors
        .iter()
        .filter(|entry| entry.method.eq_ignore_ascii_case(&method))
    {
        if let Some((path_params, static_segments, segment_count)) =
            match_path_template(&descriptor.path_template, path)
        {
            candidates.push((
                MatchCandidate {
                    path_template: descriptor.path_template.clone(),
                    path_params,
                    static_segments,
                    segment_count,
                },
                descriptor,
            ));
        }
    }

    if candidates.is_empty() {
        return Ok(None);
    }

    candidates.sort_by(|(left_candidate, _), (right_candidate, _)| {
        right_candidate
            .static_segments
            .cmp(&left_candidate.static_segments)
            .then_with(|| {
                right_candidate
                    .segment_count
                    .cmp(&left_candidate.segment_count)
            })
    });

    let (best_candidate, best_descriptor) = candidates.remove(0);
    if let Some((next_candidate, _)) = candidates.first() {
        if next_candidate.static_segments == best_candidate.static_segments
            && next_candidate.segment_count == best_candidate.segment_count
            && next_candidate.path_template != best_candidate.path_template
        {
            return Err(GatewayRouteResolutionError::AmbiguousTemplateMatch);
        }
    }

    Ok(Some(GatewayRouteMatch {
        metadata: GatewayRouteMetadata {
            path_template: best_descriptor.path_template.clone(),
            path_params: best_candidate.path_params,
            idempotency_semantics: best_descriptor.idempotency_semantics.clone(),
            transaction_boundary: best_descriptor.transaction_boundary,
            expected_event_emissions: best_descriptor.expected_event_emissions.clone(),
        },
    }))
}

fn match_path_template(
    template: &str,
    path: &str,
) -> Option<(BTreeMap<String, String>, usize, usize)> {
    let template_parts = split_segments(template);
    let path_parts = split_segments(path);
    if template_parts.len() != path_parts.len() {
        return None;
    }

    let mut params = BTreeMap::new();
    let mut static_segments = 0usize;
    for (template_segment, path_segment) in template_parts.iter().zip(path_parts.iter()) {
        if let Some(param_name) = template_segment.strip_prefix(':') {
            if param_name.is_empty() {
                return None;
            }
            params.insert(param_name.to_string(), (*path_segment).to_string());
        } else if template_segment == path_segment {
            static_segments += 1;
        } else {
            return None;
        }
    }
    Some((params, static_segments, template_parts.len()))
}

fn split_segments(path: &str) -> Vec<&str> {
    path.trim_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn descriptor(method: &str, path_template: &str) -> GatewayRouteDescriptor {
        GatewayRouteDescriptor {
            method: method.to_string(),
            path_template: path_template.to_string(),
            idempotency_semantics: GatewayIdempotencySemantics::NotApplicable,
            transaction_boundary: GatewayTransactionBoundary::ReadOnly,
            expected_event_emissions: Vec::new(),
        }
    }

    #[test]
    fn template_matches_with_single_param() {
        let routes = vec![descriptor("GET", "/api/items/:id")];
        let resolved = resolve_route(&routes, "GET", "/api/items/abc")
            .unwrap()
            .expect("route should resolve");
        assert_eq!(resolved.metadata.path_template, "/api/items/:id");
        assert_eq!(
            resolved.metadata.path_params.get("id").map(String::as_str),
            Some("abc")
        );
    }

    #[test]
    fn template_matches_with_two_params() {
        let routes = vec![descriptor("GET", "/api/items/:id/revisions/:revision_id")];
        let resolved = resolve_route(&routes, "GET", "/api/items/a1/revisions/r2")
            .unwrap()
            .expect("route should resolve");
        assert_eq!(
            resolved.metadata.path_params.get("id").map(String::as_str),
            Some("a1")
        );
        assert_eq!(
            resolved
                .metadata
                .path_params
                .get("revision_id")
                .map(String::as_str),
            Some("r2")
        );
    }

    #[test]
    fn static_route_wins_over_param_route() {
        let routes = vec![
            descriptor("GET", "/api/items/:id"),
            descriptor("GET", "/api/items/special"),
        ];
        let resolved = resolve_route(&routes, "GET", "/api/items/special")
            .unwrap()
            .expect("route should resolve");
        assert_eq!(resolved.metadata.path_template, "/api/items/special");
        assert!(resolved.metadata.path_params.is_empty());
    }

    #[test]
    fn ambiguous_templates_are_rejected() {
        let routes = vec![
            descriptor("GET", "/api/:scope/:id"),
            descriptor("GET", "/api/:kind/:name"),
        ];
        let err = resolve_route(&routes, "GET", "/api/system/123").unwrap_err();
        assert_eq!(err, GatewayRouteResolutionError::AmbiguousTemplateMatch);
    }
}
