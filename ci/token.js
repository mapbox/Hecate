#!/usr/bin/env node
'use strict';

const fs = require('fs');
const AWS = require('aws-sdk');
const { App } = require("@octokit/app");
const Octokit = require("@octokit/rest");

const BINARY_PATH = './target/release/hecate';

async function getCredentials(secretPrefix) {
    const sm = new AWS.SecretsManager({ region: 'us-east-1' });

    const [
        { SecretString: id },
        { SecretString: installationId },
        { SecretBinary: privateKey }
    ] = await Promise.all([
        sm.getSecretValue({ SecretId: `${secretPrefix}/app-id` }).promise(),
        sm.getSecretValue({ SecretId: `${secretPrefix}/installation-id` }).promise(),
        sm.getSecretValue({ SecretId: `${secretPrefix}/secret` }).promise()
    ]);

    return { id, installationId, privateKey };
}

async function uploadBinary(octokit, uploadUrl) {
    return octokit.repos.uploadReleaseAsset({
        name: 'hecate',
        file: fs.createReadStream(BINARY_PATH),
        url: uploadUrl,
        headers: {
            'content-length': fs.statSync(BINARY_PATH).size,
            'content-type': 'application/octet-stream'
        }
    });
}

async function createRelease(octokit) {
    const res = await octokit.repos.createRelease({
        owner: 'mapbox',
        repo: 'Hecate',
        tag_name: process.env.CIRCLE_TAG || '0.74.2-binaries',
        prerelease: true
    });

    return res;
}

(async function() {
    const secretPrefix = process.argv[2];
    if (!secretPrefix) throw new Error('Prefix value required.');

    try {
        const { id, installationId, privateKey } = await getCredentials(secretPrefix);

        const app = new App({ id, privateKey });
        const octokit = new Octokit({
            async auth() {
                const installationAccessToken = await app.getInstallationAccessToken({ installationId });
                return `token ${installationAccessToken}`;
            }
        });

        const { data: { uploadUrl } } = await createRelease(octokit);
        await uploadBinary(octokit, uploadUrl);
    } catch (err) {
        console.error(err);
        process.exit(1);
    }
})();
