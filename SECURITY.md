# Security Policy

GModPatchTool's lead maintainer, [@WinterPhoenix](https://github.com/WinterPhoenix), handles security reports.

## Reporting a Vulnerability

Please **don't** open a public issue about a vulnerability. Report it privately with the **Report a vulnerability** button on our [Advisories page](https://github.com/solsticegamestudios/GModPatchTool/security/advisories), or by emailing **security@solsticegamestudios.com** (encrypted with our [PGP key](https://solsticegamestudios.com/pgp-keys/publickey.security@solsticegamestudios.com-08d0c663fb90f28179527fbb9b54ee322104b573.asc) if you can).

Include:

- What's affected: the tool, the patch files, our CEF builds, or our gmod-html fork
- Your OS and GModPatchTool version
- How to reproduce the vulnerability
- What an attacker could do

## What Happens Next

We'll reply within a week to let you know we got your report. We're a small team, so the fix itself might take longer.

Once we've confirmed the vulnerability, we'll work on a fix with you in a private GitHub security advisory. When the fix is out, we'll publish the advisory and credit you in it (unless you'd rather stay anonymous). If we decide your report isn't a vulnerability, we'll tell you why.

Please don't share the details publicly until the fix is out.

## What's Covered

- GModPatchTool itself (this repository)
- The patch files it downloads (the [`files` branch](https://github.com/solsticegamestudios/GModPatchTool/tree/files))
- Our CEF builds and [our fork of gmod-html](https://github.com/solsticegamestudios/gmod-html)

Vulnerabilities in stock Garry's Mod are Facepunch's to fix, so report those to Facepunch. Report vulnerabilities in Chromium or CEF to those projects as well.

## Supported Versions

We only fix the latest release. If you're on an older release, update first and check if the vulnerability is still there.
