provider "aws" {
  region = var.aws_region
}

resource "aws_iam_role" "lambda_role" {
  name = "lambda_role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "lambda.amazonaws.com"
        }
      }
    ]
  })
}

# Find a certificate that is issued
data "aws_acm_certificate" "issued" {
  domain   = "{domain_name}"
  statuses = ["ISSUED"]
}

resource "aws_api_gateway_domain_name" "custom_domain" {
  domain_name              = "{domain_name}"
  regional_certificate_arn = data.aws_acm_certificate.issued.arn
  endpoint_configuration {
    types = ["REGIONAL"]
  }
}

resource "aws_iam_role_policy_attachment" "lambda_policy_attachment" {
  role       = aws_iam_role.lambda_role.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

resource "aws_api_gateway_rest_api" "api_gateway" {
  name = var.api_gateway_name
}

resource "aws_api_gateway_base_path_mapping" "custom_mapping" {
  domain_name = aws_api_gateway_domain_name.custom_domain.domain_name
  api_id      = aws_api_gateway_rest_api.api_gateway.id
  stage_name  = var.api_gateway_stage_name
}
