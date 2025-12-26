resource "aws_lambda_function" "flight_api" {
  function_name = "flight_api"
  role          = aws_iam_role.lambda_exec.arn
  handler       = "bootstrap"
  runtime       = "provided.al2023"
  architectures = ["arm64"]
  memory_size   = 128
  timeout       = 10

  filename = "../target/lambda/rupeetravel-api/bootstrap.zip"
  
  # This is needed because the filename might not exist during plan if not built yet, 
  # but Terraform requires it for hashing. 
  # We assume the user builds it before apply.
  source_code_hash = fileexists("../target/lambda/rupeetravel-api/bootstrap.zip") ? filebase64sha256("../target/lambda/rupeetravel-api/bootstrap.zip") : null


  environment {
    variables = {
      S3_BUCKET           = aws_s3_bucket.flights_db.bucket
      FLIGHT_API_PASSWORD = var.flight_api_password
      RUST_BACKTRACE      = "1"
    }
  }
}
