//! The AWS console services granted understands via `assume -s <alias>`.
//!
//! This mirrors granted's own `ServiceMap` (`pkg/console/service_map.go`): each
//! entry maps a CLI alias to the AWS console destination it opens (e.g. `ec2` →
//! `ec2/v2`, `ddb` → `dynamodbv2`). It is a **static snapshot** compiled into the
//! binary — granted's map is a fixed table, not runtime data — captured from
//! granted v0.39.0. When bumping granted, re-check that file and update
//! [`SERVICES`] to match. granted's empty-alias default (`"" -> console`) is
//! intentionally omitted: an empty service query is handled as "plain console".

/// A console service alias understood by granted, with the destination it opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Service {
    /// The alias passed to `assume -s` (e.g. `ec2`, `ddb`).
    pub alias: &'static str,
    /// The AWS console destination it opens (e.g. `ec2/v2`, `dynamodbv2`).
    pub destination: &'static str,
}

/// Every alias granted accepts, sorted by alias. Mirrors granted's `ServiceMap`.
pub const SERVICES: &[Service] = &[
    Service {
        alias: "acm",
        destination: "acm",
    },
    Service {
        alias: "aos",
        destination: "aos",
    },
    Service {
        alias: "apigateway",
        destination: "apigateway",
    },
    Service {
        alias: "apigw",
        destination: "apigateway",
    },
    Service {
        alias: "appsync",
        destination: "appsync",
    },
    Service {
        alias: "athena",
        destination: "athena",
    },
    Service {
        alias: "backup",
        destination: "backup",
    },
    Service {
        alias: "bedrock",
        destination: "bedrock",
    },
    Service {
        alias: "billing",
        destination: "billing",
    },
    Service {
        alias: "c9",
        destination: "cloud9",
    },
    Service {
        alias: "ce",
        destination: "cost-management",
    },
    Service {
        alias: "cf",
        destination: "cloudfront",
    },
    Service {
        alias: "cfn",
        destination: "cloudformation",
    },
    Service {
        alias: "cloudformation",
        destination: "cloudformation",
    },
    Service {
        alias: "cloudfront",
        destination: "cloudfront",
    },
    Service {
        alias: "cloudmap",
        destination: "cloudmap",
    },
    Service {
        alias: "cloudwatch",
        destination: "cloudwatch",
    },
    Service {
        alias: "codeartifact",
        destination: "codesuite/codeartifact",
    },
    Service {
        alias: "codecommit",
        destination: "codesuite/codecommit",
    },
    Service {
        alias: "codedeploy",
        destination: "codedeploy",
    },
    Service {
        alias: "codepipeline",
        destination: "codepipeline",
    },
    Service {
        alias: "codesuite",
        destination: "codesuite",
    },
    Service {
        alias: "cognito",
        destination: "cognito",
    },
    Service {
        alias: "config",
        destination: "config",
    },
    Service {
        alias: "controltower",
        destination: "controltower",
    },
    Service {
        alias: "ct",
        destination: "cloudtrail",
    },
    Service {
        alias: "cw",
        destination: "cloudwatch",
    },
    Service {
        alias: "ddb",
        destination: "dynamodbv2",
    },
    Service {
        alias: "dms",
        destination: "dms/v2",
    },
    Service {
        alias: "dx",
        destination: "directconnect/v2",
    },
    Service {
        alias: "dynamodb",
        destination: "dynamodbv2",
    },
    Service {
        alias: "eb",
        destination: "elasticbeanstalk",
    },
    Service {
        alias: "ebs",
        destination: "elasticbeanstalk",
    },
    Service {
        alias: "ec2",
        destination: "ec2/v2",
    },
    Service {
        alias: "ecr",
        destination: "ecr",
    },
    Service {
        alias: "ecs",
        destination: "ecs",
    },
    Service {
        alias: "eks",
        destination: "eks",
    },
    Service {
        alias: "elasticache",
        destination: "elasticache",
    },
    Service {
        alias: "eventbridge",
        destination: "events",
    },
    Service {
        alias: "events",
        destination: "events",
    },
    Service {
        alias: "ga",
        destination: "globalaccelerator",
    },
    Service {
        alias: "gd",
        destination: "guardduty",
    },
    Service {
        alias: "globalaccelerator",
        destination: "globalaccelerator",
    },
    Service {
        alias: "grafana",
        destination: "grafana",
    },
    Service {
        alias: "iam",
        destination: "iamv2",
    },
    Service {
        alias: "kms",
        destination: "kms",
    },
    Service {
        alias: "l",
        destination: "lambda",
    },
    Service {
        alias: "lambda",
        destination: "lambda",
    },
    Service {
        alias: "mwaa",
        destination: "mwaa",
    },
    Service {
        alias: "organizations",
        destination: "organizations/v2",
    },
    Service {
        alias: "orgs",
        destination: "organizations/v2",
    },
    Service {
        alias: "param",
        destination: "systems-manager/parameters",
    },
    Service {
        alias: "r53",
        destination: "route53/v2",
    },
    Service {
        alias: "ram",
        destination: "ram",
    },
    Service {
        alias: "rds",
        destination: "rds",
    },
    Service {
        alias: "redshift",
        destination: "redshiftv2",
    },
    Service {
        alias: "route53",
        destination: "route53/v2",
    },
    Service {
        alias: "s3",
        destination: "s3",
    },
    Service {
        alias: "sagemaker",
        destination: "sagemaker",
    },
    Service {
        alias: "scrh",
        destination: "securityhub",
    },
    Service {
        alias: "scrm",
        destination: "secretsmanager",
    },
    Service {
        alias: "secretsmanager",
        destination: "secretsmanager",
    },
    Service {
        alias: "securityhub",
        destination: "securityhub",
    },
    Service {
        alias: "ses",
        destination: "ses",
    },
    Service {
        alias: "sfn",
        destination: "states",
    },
    Service {
        alias: "sm",
        destination: "secretsmanager",
    },
    Service {
        alias: "sns",
        destination: "sns",
    },
    Service {
        alias: "sqs",
        destination: "sqs",
    },
    Service {
        alias: "ssm",
        destination: "systems-manager",
    },
    Service {
        alias: "sso",
        destination: "singlesignon",
    },
    Service {
        alias: "states",
        destination: "states",
    },
    Service {
        alias: "stepfn",
        destination: "states",
    },
    Service {
        alias: "tra",
        destination: "trustedadvisor",
    },
    Service {
        alias: "trustedadvisor",
        destination: "trustedadvisor",
    },
    Service {
        alias: "vpc",
        destination: "vpc",
    },
    Service {
        alias: "waf",
        destination: "wafv2",
    },
    Service {
        alias: "wafv2",
        destination: "wafv2/homev2",
    },
];

/// Filter [`SERVICES`] against `query`, keeping only entries that contain every
/// whitespace-separated term of the query as a case-insensitive substring. Each
/// term is matched against both the alias and the destination, so `dynamo`
/// finds `ddb`/`dynamodb` and `cost` finds `ce` (→ `cost-management`). The
/// alphabetical-by-alias order is preserved. An empty query returns everything.
pub fn filter(query: &str) -> Vec<Service> {
    let terms: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();

    if terms.is_empty() {
        return SERVICES.to_vec();
    }

    SERVICES
        .iter()
        .copied()
        .filter(|service| {
            terms
                .iter()
                .all(|term| service.alias.contains(term) || service.destination.contains(term))
        })
        .collect()
}
