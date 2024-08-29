variable "aws_region" {
  description = "AWS region"
  type        = string
  default     = "us-east-1"
}

variable "lambda_function_name" {
  description = "The name of the lambda function"
  type        = string
  default     = "node_lambda"
}

variable "lambda_runtime" {
  description = "Which runtime the lambda should use (e.g. nodejs20.x)"
  type        = string
  default     = "nodejs20.x"
}

variable "lambda_environment_variables" {
  description = "Environment variables passed to the lambda script"
  type        = map(string)
  default = {
    "ENV_VAR" = "value"
  }
}

variable "api_gateway_name" {
  description = "Name of the API gateway"
  type        = string
  default     = "lambda_api"
}

variable "api_gateway_resource_path" {
  description = "The endpoint the URL should use (url/{endpoint})"
  type        = string
  default     = "endpoint"
}

variable "api_gateway_stage_name" {
  description = "The stage name of the API gateway deployment"
  type        = string
  default     = "prod"
}

variable "http_method" {
  description = "GET/POST/etc"
  type        = string
  default     = "GET"
}
