resource "aws_lambda_function" "node_lambda" {
  function_name = "{function_name}" # var.lambda_function_name
  role          = aws_iam_role.lambda_role.arn
  handler       = "index.handler"
  runtime       = "{runtime}" # var.lambda_runtime

  filename = "lambda/lambda_function.zip"

  source_code_hash = filebase64sha256("lambda/lambda_function.zip")

  environment {
    variables = var.lambda_environment_variables # {environment_variables} # 
  }
}

resource "aws_api_gateway_method" "api_method" {
  rest_api_id   = aws_api_gateway_rest_api.api_gateway.id
  resource_id   = aws_api_gateway_resource.api_resource.id
  http_method   = "{http_method}" # var.http_method
  authorization = "NONE"
}

resource "aws_api_gateway_resource" "api_resource" {
  rest_api_id = aws_api_gateway_rest_api.api_gateway.id
  parent_id   = aws_api_gateway_rest_api.api_gateway.root_resource_id
  path_part   = "{resource_path}" # var.api_gateway_resource_path # "endpoint" # Change path as needed
}

resource "aws_api_gateway_integration" "lambda_integration" {
  rest_api_id = aws_api_gateway_rest_api.api_gateway.id
  resource_id = aws_api_gateway_resource.api_resource.id
  http_method = aws_api_gateway_method.api_method.http_method

  integration_http_method = "POST" # lambda can only be invoked w/ POST requests, so this turns the "GET" into a "POST" the lambda can handle
  type                    = "AWS_PROXY"
  uri                     = aws_lambda_function.node_lambda.invoke_arn
}

resource "aws_lambda_permission" "api_gateway_permission" {
  statement_id  = "AllowAPIGatewayInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.node_lambda.function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_api_gateway_rest_api.api_gateway.execution_arn}/*/*"
}

resource "aws_api_gateway_deployment" "api_deployment" {
  depends_on = [aws_api_gateway_integration.lambda_integration]

  rest_api_id = aws_api_gateway_rest_api.api_gateway.id
  stage_name  = var.api_gateway_stage_name # should use aws_api_gateway_stage resource instead.
}

output "api_url" {
  value = "${aws_api_gateway_deployment.api_deployment.invoke_url}/${var.api_gateway_resource_path}"
}

