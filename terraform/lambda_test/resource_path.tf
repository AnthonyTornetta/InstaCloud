resource "aws_api_gateway_resource" "api_resource_{resource_path}" {
  rest_api_id = aws_api_gateway_rest_api.api_gateway.id
  parent_id   = aws_api_gateway_rest_api.api_gateway.root_resource_id
  path_part   = "{resource_path}"
}

