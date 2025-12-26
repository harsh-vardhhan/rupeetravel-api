resource "aws_apigatewayv2_api" "flight_api" {
  name          = "rupeetravel-api"
  protocol_type = "HTTP"
}

resource "aws_apigatewayv2_stage" "default" {
  api_id      = aws_apigatewayv2_api.flight_api.id
  name        = "$default"
  auto_deploy = true
}

resource "aws_apigatewayv2_integration" "lambda_integration" {
  api_id           = aws_apigatewayv2_api.flight_api.id
  integration_type = "AWS_PROXY"

  connection_type      = "INTERNET"
  integration_method   = "POST"
  integration_uri      = aws_lambda_function.flight_api.invoke_arn
  payload_format_version = "2.0"
}

# Removed specific route to match SAM template which only has root catch-all

# IMPORTANT: Route for the exact path if strictly matching original template which was /{proxy+}
# Original template: Path: /{proxy+}. Wait, that probably meant root.
# Actually, the Output says: https://.../api/flights
# The template output value is: !Sub "https://${ServerlessHttpApi}.execute-api.${AWS::Region}.amazonaws.com/api/flights"
# But the Event path is `/{proxy+}`.
# This implies the Base URL of the API gives you the domain, and the user appends `/api/flights`.
# If the SAM template defines the event path as `/{proxy+}` on the function, then ANY request to `/<anything>` goes to lambda.
# However, the OUTPUT says `/api/flights`.
# If the user wants the API to be at `/api/flights`, then the route key should be `ANY /api/flights/{proxy+}` or `ANY /api/flights`.
# Let's look at template again.
# Path: /{proxy+}
# This means the API handles everything under root.
# The Output `ApiUrl` appends `/api/flights`. This suggests the CLIENT usage, not the infrastructure constraint.
# Use `ANY /{proxy+}` to match the exact SAM behavior.

resource "aws_apigatewayv2_route" "default_proxy" {
  api_id    = aws_apigatewayv2_api.flight_api.id
  route_key = "ANY /{proxy+}"
  target    = "integrations/${aws_apigatewayv2_integration.lambda_integration.id}"
}


resource "aws_lambda_permission" "api_gw" {
  statement_id  = "AllowExecutionFromAPIGateway"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.flight_api.function_name
  principal     = "apigateway.amazonaws.com"

  source_arn = "${aws_apigatewayv2_api.flight_api.execution_arn}/*/*"
}
