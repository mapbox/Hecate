<h1 align='center'>Migrations</h1>

This directory contains database migrations. All migrations should be:
- in `.sql` files
- named `<hecate version to be released>-<pre|post>`, e.g. `v0.80.0-post.sql`. Name migrations with `pre` if they're to be run before the new Hecate binary is to be deployed. Named migrations with `pro` if they're to be run after deploy.
- include a comment at the top describing what the migration does
