output "api_endpoint" {
  description = "API Gateway endpoint URL"
  # Match the format of SAM output: https://{api_id}.execute-api.{region}.amazonaws.com/api/flights
  # Note: The terraform resource invoke_url already includes the protocol.
  # But we want to append /api/flights to match the SAM output display value if desired, or just give the base.
  # SAM Output: !Sub "https://${ServerlessHttpApi}.execute-api.${AWS::Region}.amazonaws.com/api/flights"
  # We will output the base URL and the suggested full URL.
  value = "${aws_apigatewayv2_api.flight_api.api_endpoint}/api/flights"
}

output "bucket_name" {
  description = "S3 Bucket Name"
  value       = aws_s3_bucket.flights_db.bucket
}
