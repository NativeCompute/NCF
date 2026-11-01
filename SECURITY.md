# Security

Please do not open a public issue for a security problem.

Report it privately through GitHub: the repository's **Security** tab → **Report a vulnerability**. If that is not available to you, email **hello@nativecompute.com** with "NCF security" in the subject.

Tell us what you found, how to reproduce it, and which version you used. We will acknowledge the report within a few days, keep you posted while we fix it, and credit you in the release notes if you would like.

NCF reaches the network in one place: the update check in `crates/ncf/src/update.rs`, which reads `https://api.github.com/repos/<repo>/releases/latest` over HTTPS and can be switched off in Settings → About.
