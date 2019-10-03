module.exports = {
    "Outputs": {
        "AccessKeyId": {
            "Value": {
                "Ref": "AccessKey"
            }
        },
        "SecretAccessKey": {
            "Value": {
                "Fn::GetAtt": [
                    "AccessKey",
                    "SecretAccessKey"
                ]
            }
        }
    },
    "Parameters": {
        "SecretPrefix": {
            "Description": "Prefix of GitHub App",
            "Type": "String"
        }
    },
    "Resources": {
        "AccessKey": {
            "Properties": {
                "UserName": {
                    "Ref": "User"
                }
            },
            "Type": "AWS::IAM::AccessKey"
        },
        "User": {
            "Properties": {
                "Policies": [
                    {
                        "PolicyDocument": {
                            "Statement": [
                                {
                                    "Action": "secretsmanager:GetSecretValue",
                                    "Effect": "Allow",
                                    "Resource": {
                                        "Fn::Join": [
                                            ":",
                                            [
                                                "arn:aws:secretsmanager",
                                                {
                                                    "Ref": "AWS::Region"
                                                },
                                                {
                                                    "Ref": "AWS::AccountId"
                                                },
                                                "secret",
                                                {
                                                    "Fn::Join": [
                                                        "",
                                                        [
                                                            {
                                                                "Ref": "SecretPrefix"
                                                            },
                                                            "/*"
                                                        ]
                                                    ]
                                                }
                                            ]
                                        ]
                                    }
                                }
                            ]
                        },
                        "PolicyName": "github-apps"
                    }
                ]
            },
            "Type": "AWS::IAM::User"
        }
    }
}
