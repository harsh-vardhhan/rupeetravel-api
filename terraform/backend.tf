terraform {
  backend "s3" {
    bucket         = "terraform-state-479650866443-ap-south-1"
    key            = "rupeetravel-api/terraform.tfstate"
    region         = "ap-south-1"
    dynamodb_table = "terraform-locks"
    encrypt        = true
  }
}
