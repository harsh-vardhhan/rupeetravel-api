data "aws_caller_identity" "current" {}
data "aws_region" "current" {}

resource "aws_s3_bucket" "flights_db" {
  bucket = "flights-db-${data.aws_caller_identity.current.account_id}-${data.aws_region.current.name}"
}
