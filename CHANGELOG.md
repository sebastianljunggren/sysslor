# Changelog

## [1.0.0](https://github.com/sebastianljunggren/sysslor/compare/v0.6.0...v1.0.0) (2026-11-01)


### ⚠ BREAKING CHANGES

* `SYSSLOR_TIME_ZONE` no longer defaults to `Europe/Stockholm`. Set it to your IANA time zone, e.g. `SYSSLOR_TIME_ZONE=Europe/Stockholm`.

### Features

* require SYSSLOR_TIME_ZONE ([b7158b0](https://github.com/sebastianljunggren/sysslor/commit/b7158b0ca3b7d65ea133ed7267273127d64dc8cd))

## [0.6.0](https://github.com/sebastianljunggren/sysslor/compare/v0.5.0...v0.6.0) (2026-10-28)


### Features

* add routing and migrate to SvelteKit ([#78](https://github.com/sebastianljunggren/sysslor/issues/78)) ([0fbe720](https://github.com/sebastianljunggren/sysslor/commit/0fbe7208fd054bd302adf8ff2b4f7168cf871767))
* move navigation to a sidebar ([#76](https://github.com/sebastianljunggren/sysslor/issues/76)) ([ff20103](https://github.com/sebastianljunggren/sysslor/commit/ff20103b077fbafbeb1d5b58503bd9d9627a3ae5))
* rename edit to manage and structure it with tabs ([#73](https://github.com/sebastianljunggren/sysslor/issues/73)) ([83b5820](https://github.com/sebastianljunggren/sysslor/commit/83b582089a59126ef631339dc5fbaa4e8b5a4927))

## [0.5.0](https://github.com/sebastianljunggren/sysslor/compare/v0.4.0...v0.5.0) (2026-10-18)


### Features

* filter groups with a select ([#53](https://github.com/sebastianljunggren/sysslor/issues/53)) ([eff4526](https://github.com/sebastianljunggren/sysslor/commit/eff4526d6781e4bb9247a6732cb6b6c71bef212a))


### Bug Fixes

* close the event stream while the page is hidden ([#54](https://github.com/sebastianljunggren/sysslor/issues/54)) ([e55d6f2](https://github.com/sebastianljunggren/sysslor/commit/e55d6f2fcf9cff852cf81cfd16f6b715f6994340))

## [0.4.0](https://github.com/sebastianljunggren/sysslor/compare/v0.3.0...v0.4.0) (2026-10-11)


### Features

* show due dates and completions as relative times ([#28](https://github.com/sebastianljunggren/sysslor/issues/28)) ([abda800](https://github.com/sebastianljunggren/sysslor/commit/abda800ed03f00d8b33d3fceb9004f18c2987f00))

## [0.3.0](https://github.com/sebastianljunggren/sysslor/compare/v0.2.0...v0.3.0) (2026-10-10)


### Features

* move executor picker to modal ([#21](https://github.com/sebastianljunggren/sysslor/issues/21)) ([171229f](https://github.com/sebastianljunggren/sysslor/commit/171229f3bc6b7433accf1e670ad64b47f5bc674a))

## [0.2.0](https://github.com/sebastianljunggren/sysslor/compare/v0.1.0...v0.2.0) (2026-10-06)


### Features

* add collation support and standardize names of config variables ([0eab305](https://github.com/sebastianljunggren/sysslor/commit/0eab30526e12b1fc7f5173401b52d46232bac7a7))
* add db schema and API ([3fd4b96](https://github.com/sebastianljunggren/sysslor/commit/3fd4b96689a26d51cf36ce0529e880a185570d62))
* add grouping support for tasks ([3f5519c](https://github.com/sebastianljunggren/sysslor/commit/3f5519c123ff6c1625ebfbbac2a135d02c94e7e3))
* add optional colors to groups ([066ce0b](https://github.com/sebastianljunggren/sysslor/commit/066ce0b9a8e4881e615a76011ad16a51948f8d2c))
* add skeleton and domain for backend ([507b930](https://github.com/sebastianljunggren/sysslor/commit/507b9308dbac3e9a065a0cfda6cf860382e8a664))
* add skeleton for UI ([7ad01b0](https://github.com/sebastianljunggren/sysslor/commit/7ad01b0a2dc22ebdd34eb192af275dafbbf15111))
* add support for auth with and admin password ([dccb1a9](https://github.com/sebastianljunggren/sysslor/commit/dccb1a98c074331054cfba10a4cfa02a6a8b6ec5))
* add support for editing project name ([74103c4](https://github.com/sebastianljunggren/sysslor/commit/74103c4945e60f221a11bfe1e0d95b2a2fca2102))
* Add support for i18n with English and Swedish support ([b2e65fc](https://github.com/sebastianljunggren/sysslor/commit/b2e65fc0c26417a301e30dac2d8ad813dee9f185))
* add support for SSE ([13709eb](https://github.com/sebastianljunggren/sysslor/commit/13709ebc45a9c16ea99969d72c31ef875663d611))
* fill group pill with its color ([bb9026b](https://github.com/sebastianljunggren/sysslor/commit/bb9026ba38dd6e2aabec6e65a73ccd9283dd2640))
* implement first version of UI ([da7eee6](https://github.com/sebastianljunggren/sysslor/commit/da7eee6b01910d9e7813165dc228b9c331bc0d46))
* use color scheme based on solarized ([255c465](https://github.com/sebastianljunggren/sysslor/commit/255c465f74c8c17be2947e18db3ab4256a5d74f7))


### Bug Fixes

* remove connection error on login ([4c5c2a8](https://github.com/sebastianljunggren/sysslor/commit/4c5c2a8536b71e54475e31211aa168bbc6776b6b))
