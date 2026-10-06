# Sysslor

Sysslor is a chore list where every task has a cadence. Complete "Vacuum" with a cadence of
7 days and it comes back a week later. Tasks that need attention, because they are overdue or
prioritized, are sorted to the top. Changes show up live on every open device. It is built for
one family sharing a single password, and ships as a single small container image backed by
SQLite.

## Running

```sh
podman run -d --name sysslor \
  -p 8080:8080 \
  -v sysslor-data:/data:U \
  -e SYSSLOR_ADMIN_PASSWORD='choose-a-password' \
  -e SYSSLOR_COOKIE_KEY="$(openssl rand -base64 48)" \
  ghcr.io/sebastianljunggren/sysslor:latest
```

The container runs as UID 65532, so `/data` must be writable by that user. With Podman, `:U`
takes care of it. With Docker, `chown` the volume or bind mount to `65532:65532` first.

Keep `SYSSLOR_COOKIE_KEY` stable across restarts, or everyone gets logged out.

## Configuration

All configuration is read from environment variables.

| Variable                  | Default                                          | Description                                                                                         |
| ------------------------- | ------------------------------------------------ | --------------------------------------------------------------------------------------------------- |
| `SYSSLOR_ADMIN_PASSWORD`  | required                                         | The shared login password.                                                                          |
| `SYSSLOR_COOKIE_KEY`      | required                                         | Key for signing session cookies, at least 32 bytes. Generate one with `openssl rand -base64 48`.    |
| `SYSSLOR_TIME_ZONE`       | `Europe/Stockholm`                               | Time zone used to decide which calendar day a task is due.                                          |
| `SYSSLOR_LOCALE`          | `en`                                             | Language used to sort names (groups, tasks). This is the collation language, not the UI language.   |
| `SYSSLOR_BIND_ADDR`       | `0.0.0.0:8080`                                   | Address to listen on.                                                                               |
| `SYSSLOR_DATABASE_URL`    | `sqlite:///data/sysslor.db` in the image         | SQLite database location.                                                                           |

## Deployment notes

- Serve it over HTTPS (or `localhost`). The session cookie is `Secure`, so logging in over plain
  HTTP will not work.
- Run a single instance. SQLite cannot have two writers, so use one replica and stop the old
  container before starting a new one.
- Disable proxy buffering for `/api/events`, which is a Server-Sent Events stream used for
  live updates.
- Put the database on local or block storage, not NFS. SQLite file locking does not work
  reliably on NFS.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion
in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above,
without any additional terms or conditions.
