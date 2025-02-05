resource "aws_lambda_function" "node_lambda_{api_identifier}_{function_name}" {
  function_name = "{api_identifier}_{function_name}" # var.lambda_function_name
  role          = aws_iam_role.lambda_role.arn
  handler       = "index.handler"
  runtime       = "{runtime}" # var.lambda_runtime

  filename = "lambda_function_{api_identifier}_{function_name}.zip"

  source_code_hash = filebase64sha256("lambda_function_{api_identifier}_{function_name}.zip")

  environment {
    variables = {environment_variables}
  }
}

resource "aws_api_gateway_method" "api_method_{api_identifier}_{function_name}" {
  rest_api_id   = aws_api_gateway_rest_api.api_gateway.id
  resource_id   = aws_api_gateway_resource.api_resource_{resource_path_hash}.id
  http_method   = "{http_method}" # var.http_method
  authorization = "NONE"
}

# TODO: I don't think the "depends_on" in this is needed.
resource "aws_api_gateway_integration" "lambda_integration_{api_identifier}_{function_name}" {
  rest_api_id = aws_api_gateway_rest_api.api_gateway.id
  resource_id = aws_api_gateway_resource.api_resource_{resource_path_hash}.id
  http_method = aws_api_gateway_method.api_method_{api_identifier}_{function_name}.http_method

  depends_on = [aws_lambda_function.node_lambda_{api_identifier}_{function_name}]
  
  integration_http_method = "POST" # lambda can only be invoked w/ POST requests, so this turns the "GET" into a "POST" the lambda can handle
  type                    = "AWS_PROXY"
  uri                     = aws_lambda_function.node_lambda_{api_identifier}_{function_name}.invoke_arn
}

resource "aws_lambda_permission" "api_gateway_permission_{api_identifier}_{function_name}" {
  statement_id  = "AllowAPIGatewayInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.node_lambda_{api_identifier}_{function_name}.function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_api_gateway_rest_api.api_gateway.execution_arn}/*/*/*"
}

resource "aws_api_gateway_deployment" "api_deployment_{api_identifier}_{function_name}" {
  depends_on = [
    {depends_on}
  ] # [aws_api_gateway_integration.lambda_integration_{api_identifier}_{function_name}]

  rest_api_id = aws_api_gateway_rest_api.api_gateway.id
  # https://stackoverflow.com/questions/48955987/missing-authentication-token-on-unauthenticated-method
  # This being a fixed thing is preventing re-applies from working properly
  stage_name  = var.api_gateway_stage_name # should use aws_api_gateway_stage resource instead.
}

output "api_url_{api_identifier}_{function_name}" {
  value = "${aws_api_gateway_deployment.api_deployment_{api_identifier}_{function_name}.invoke_url}/{resource_path}"
}

output "custom_api_url_{api_identifier}_{function_name}" {
  value = "https://${aws_api_gateway_domain_name.custom_domain.domain_name}/{resource_path}"
}
