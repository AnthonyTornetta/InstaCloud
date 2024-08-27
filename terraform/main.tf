terraform {
  required_providers {
    aws = {
      source = "hashicorp/aws"
    }
  }
}

provider "aws" {
  region = "us-east-1"
}

resource "aws_key_pair" "my_key_pair" {
  key_name = "my_key_pair"
  public_key = file("~/.ssh/id_ras.pub")
}

resource "aws_security_group" "allow_ssh_http" {
  name = "allow_ssh_http"
  description = "Allows SSH and HTTP(S) traffic"

  ingress {
    from_port = 22
    to_port = 22
    protocol = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    from_port = 80
    to_port = 80
    protocol = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    from_port = 443
    to_port = 443
    protocol = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    from_port = 0
    to_port = 0
    protocol = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "aws_instance" "my_ec2_instance" {
  ami = "ami-0c55b159cbfafe1f0"
  instance_type = "t2.micro"

  key_name = aws_key_pair.my_key_pair.key_name
  security_groups = aws_security_group.allow_ssh_http.name

  tags = {
    Name = "MyEC2Instance"
  }
}

