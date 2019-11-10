'use strict';

const cf = require('@mapbox/cloudfriend');

module.exports = {
    Parameters: {
        SecretPrefix: {
            Type: 'String',
            Description: 'Prefix of GitHub App'
        }
    },
    Resources: {
        User: {
            Type: 'AWS::IAM::User',
            Properties: {
                Policies: [
                    {
                        PolicyName: 'github-apps',
                        PolicyDocument: {
                            Statement: [
                                {
                                    Effect: 'Allow',
                                    Action: 'secretsmanager:GetSecretValue',
                                    Resource: cf.join(':', [
                                        'arn:aws:secretsmanager',
                                        cf.region,
                                        cf.accountId,
                                        'secret',
                                        cf.join([cf.ref('SecretPrefix'), '/*'])
                                    ])
                                }
                            ]
                        }
                    }
                ]
            }
        },
        AccessKey: {
            Type: 'AWS::IAM::AccessKey',
            Properties: {
                UserName: cf.ref('User')
            }
        }
    },
    Outputs: {
        AccessKeyId: {
            Value: cf.ref('AccessKey')
        },
        SecretAccessKey: {
            Value: cf.getAtt('AccessKey', 'SecretAccessKey')
        }
    }
};
