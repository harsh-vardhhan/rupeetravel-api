resource "aws_iam_role" "lambda_exec" {
  name = "flight_api_lambda_role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Action = "sts:AssumeRole"
      Effect = "Allow"
      Principal = {
        Service = "lambda.amazonaws.com"
      }
    }]
  })
}

resource "aws_iam_role_policy_attachment" "lambda_basic_execution" {
  role       = aws_iam_role.lambda_exec.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

resource "aws_iam_policy" "s3_crud" {
  name        = "flight_api_s3_crud"
  description = "Allow CRUD operations on Flights DB Bucket"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Action = [
        "s3:GetObject",
        "s3:PutObject",
        "s3:DeleteObject",
        "s3:ListBucket"
      ]
      Effect = "Allow"
      Resource = [
        aws_s3_bucket.flights_db.arn,
        "${aws_s3_bucket.flights_db.arn}/*"
      ]
    }]
  })
}

resource "aws_iam_role_policy_attachment" "lambda_s3_crud" {
  role       = aws_iam_role.lambda_exec.name
  policy_arn = aws_iam_policy.s3_crud.arn
}
