provider "aws" {
  region = "us-west-2"
}

resource "aws_eks_cluster" "fluxgate_cluster" {
  name     = "fluxgate-cluster"
  role_arn = aws_iam_role.eks_cluster_role.arn

  vpc_config {
    subnet_ids = [aws_subnet.main1.id, aws_subnet.main2.id, aws_subnet.main3.id]
  }

  // Enterprise Hardening: Enable envelope encryption for secrets
  encryption_config {
    resources = ["secrets"]
    provider {
      key_arn = aws_kms_key.eks_secrets.arn
    }
  }
}

resource "aws_db_instance" "fluxgate_postgres" {
  allocated_storage      = 100
  storage_type           = "gp3"
  engine                 = "postgres"
  engine_version         = "15.4"
  instance_class         = "db.t3.large"
  db_name                = "fluxgate_db"
  username               = "postgres"
  password               = var.db_password # Using var for security
  multi_az               = true            # Production High Availability
  storage_encrypted      = true
  kms_key_id             = aws_kms_key.db_encryption.arn
  backup_retention_period = 7
  skip_final_snapshot    = false
}

resource "aws_elasticache_replication_group" "fluxgate_redis_ha" {
  replication_group_id          = "fluxgate-redis-ha"
  replication_group_description = "High Availability Redis for FluxGate"
  node_type                     = "cache.t4g.small"
  num_cache_clusters            = 2
  automatic_failover_enabled    = true
  multi_az_enabled              = true
  at_rest_encryption_enabled    = true
  transit_encryption_enabled    = true
  port                          = 6379
}

resource "aws_wafv2_web_acl" "fluxgate_waf" {
  name     = "fluxgate-waf"
  scope    = "REGIONAL"
  default_action {
    allow {}
  }

  rule {
    name     = "AWSManagedRulesCommonRuleSet"
    priority = 1

    override_action {
      none {}
    }

    statement {
      managed_rule_group_statement {
        name        = "AWSManagedRulesCommonRuleSet"
        vendor_name = "AWS"
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "fluxgate-waf-common"
      sampled_requests_enabled   = true
    }
  }

  visibility_config {
    cloudwatch_metrics_enabled = true
    metric_name                = "fluxgate-waf-main"
    sampled_requests_enabled   = true
  }
}
