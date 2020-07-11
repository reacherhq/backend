# Changelog

All notable changes to this project will be documented in this file. See [standard-version](https://github.com/conventional-changelog/standard-version) for commit guidelines.

### [0.2.5](https://github.com/reacherhq/backend/compare/v0.2.4...v0.2.5) (2020-07-11)


### Bug Fixes

* Update check-if-email-exists, use proxy in Yahoo API ([#99](https://github.com/reacherhq/backend/issues/99)) ([93cc16f](https://github.com/reacherhq/backend/commit/93cc16f59b078d113900ee7c697c1066bde0ef7e))

### [0.2.4](https://github.com/reacherhq/backend/compare/v0.2.3...v0.2.4) (2020-06-30)


### Features

* Add /version to heroku ([#82](https://github.com/reacherhq/backend/issues/82)) ([c619970](https://github.com/reacherhq/backend/commit/c619970ad6a67e6b3d6faf561dacae6dd1564f71))
* Deploy to serverless ([#80](https://github.com/reacherhq/backend/issues/80)) ([cbe7220](https://github.com/reacherhq/backend/commit/cbe7220d3dab47e627458ee8eb770b7704a99520))


### Bug Fixes

* Update packages and add more Sentry error checks ([#94](https://github.com/reacherhq/backend/issues/94)) ([e1141dd](https://github.com/reacherhq/backend/commit/e1141dd5a5116af0c1cd4b11b11058741efb4c02))
* **openapi:** Add input schema, fix descriptions ([#84](https://github.com/reacherhq/backend/issues/84)) ([ddc137c](https://github.com/reacherhq/backend/commit/ddc137c305d138ac63efbc7cdc68802fb8794154))
* Better loggin for staging vs prod ([#77](https://github.com/reacherhq/backend/issues/77)) ([044b1e4](https://github.com/reacherhq/backend/commit/044b1e4c46995d374b8ddaafa91df99b41912f39))

### [0.2.3](https://github.com/reacherhq/backend/compare/v0.2.2...v0.2.3) (2020-05-30)


### Features

* Add heroku deployment ([#72](https://github.com/reacherhq/backend/issues/72)) ([e08b70f](https://github.com/reacherhq/backend/commit/e08b70fa4a4d2b0d153a9200f84ac5164e0de204))


### Bug Fixes

* Add additional error message parsing ([#71](https://github.com/reacherhq/backend/issues/71)) ([8b7c394](https://github.com/reacherhq/backend/commit/8b7c394c982f6effa550284c3fbef17edc0d73a0))

### [0.2.2](https://github.com/reacherhq/backend/compare/v0.2.1...v0.2.2) (2020-05-24)


### Features

* Add success rate and verification time metrics ([#70](https://github.com/reacherhq/backend/issues/70)) ([911b9e1](https://github.com/reacherhq/backend/commit/911b9e1a0b7a32cac70b11b2f0af19fdc947b9de))

### [0.2.1](https://github.com/reacherhq/backend/compare/v0.2.0...v0.2.1) (2020-05-23)


### Bug Fixes

* Better retry mechanism, with or without Tor ([#68](https://github.com/reacherhq/backend/issues/68)) ([83fd4fe](https://github.com/reacherhq/backend/commit/83fd4fead130a1088cb23bdbc3040bd4f501efb9))
* Improve retry mechanism and error logging ([#69](https://github.com/reacherhq/backend/issues/69)) ([791da70](https://github.com/reacherhq/backend/commit/791da70a46f8a63887397435d0cc52d7c840ece2))

## [0.2.0](https://github.com/reacherhq/backend/compare/v0.1.10...v0.2.0) (2020-05-16)


### Features

* Add is_reachable field in json ([#63](https://github.com/reacherhq/backend/issues/63)) ([6fd5215](https://github.com/reacherhq/backend/commit/6fd5215285cf6b841d8c843857f9b9bf11940c82))

### [0.1.10](https://github.com/reacherhq/backend/compare/v0.1.9...v0.1.10) (2020-05-10)


### Bug Fixes

* Put correct SAASIFY_SECRET_HEADER ([#53](https://github.com/reacherhq/backend/issues/53)) ([21d0417](https://github.com/reacherhq/backend/commit/21d0417817b4c394d67ff1dd1cc48e6c8a7f50d8))

### [0.1.9](https://github.com/reacherhq/backend/compare/v0.1.8...v0.1.9) (2020-05-10)


### Features

* Add x-saasify-secret verification & retry mechanism ([#51](https://github.com/reacherhq/backend/issues/51)) ([5767e1e](https://github.com/reacherhq/backend/commit/5767e1e32497d6535ac5794a1afffbfe1cc60b05)), closes [#46](https://github.com/reacherhq/backend/issues/46) [#44](https://github.com/reacherhq/backend/issues/44)


### Bug Fixes

* Fix dockerfiles ENV ([#52](https://github.com/reacherhq/backend/issues/52)) ([c2cd1f4](https://github.com/reacherhq/backend/commit/c2cd1f42bd3d01359da9987441e05b992bdbf15c))

### [0.1.8](https://github.com/reacherhq/backend/compare/v0.1.7...v0.1.8) (2020-05-09)


### Features

* Use custom FROM email, defined in env ([#49](https://github.com/reacherhq/backend/issues/49)) ([ea31e4a](https://github.com/reacherhq/backend/commit/ea31e4abbe7e86860fbc28a4627d826afcb2b1af)), closes [#48](https://github.com/reacherhq/backend/issues/48)

### [0.1.7](https://github.com/reacherhq/backend/compare/v0.1.6...v0.1.7) (2020-05-09)


### Bug Fixes

* **deps:** Update check-if-email-exists to 0.8.1 ([#47](https://github.com/reacherhq/backend/issues/47)) ([6d83593](https://github.com/reacherhq/backend/commit/6d83593415a0956b21b6fa2e7b88b076f3bc649f))
* **openapi:** Fix outdated ref to EmailResult ([6b1615d](https://github.com/reacherhq/backend/commit/6b1615da7146232971e055d7e5fb710f585cd855))

### [0.1.6](https://github.com/reacherhq/backend/compare/v0.1.5...v0.1.6) (2020-05-08)


### Bug Fixes

* **deps:** Update to check-if-email-exists 0.8 ([#45](https://github.com/reacherhq/backend/issues/45)) ([2eaf1a2](https://github.com/reacherhq/backend/commit/2eaf1a29162a51671026156cd8be6dd592f3b76a))

### [0.1.5](https://github.com/reacherhq/backend/compare/v0.1.4...v0.1.5) (2020-05-04)


### Bug Fixes

* Fix CI building production build ([#43](https://github.com/reacherhq/backend/issues/43)) ([0a04981](https://github.com/reacherhq/backend/commit/0a04981ddc6af3b4bccf136c36bfe4dcd53b7d38))

### [0.1.4](https://github.com/reacherhq/backend/compare/v0.1.3...v0.1.4) (2020-05-04)


### Features

* Add sentry error logging ([#42](https://github.com/reacherhq/backend/issues/42)) ([37c1889](https://github.com/reacherhq/backend/commit/37c18891ccecc1b11fe306ca1bbeff7d9cd98f82))

### [0.1.3](https://github.com/reacherhq/backend/compare/v0.1.2...v0.1.3) (2020-05-04)


### Features

* Add openapi specification ([#39](https://github.com/reacherhq/backend/issues/39)) ([2c0c91d](https://github.com/reacherhq/backend/commit/2c0c91d073136bdc18f2d6d3a1ab3e60945e348f))

### [0.1.2](https://github.com/reacherhq/backend/compare/v0.1.1...v0.1.2) (2020-05-02)


### Features

* Add logging of routes ([#34](https://github.com/reacherhq/backend/issues/34)) ([3181087](https://github.com/reacherhq/backend/commit/3181087a5a627cfa13a72269f189f4e302f47e60))

### [0.1.1](https://github.com/reacherhq/backend/compare/v0.1.0...v0.1.1) (2020-05-02)


### Bug Fixes

* CI tar.gz executable file before release ([#31](https://github.com/reacherhq/backend/issues/31)) ([c1cb9c2](https://github.com/amaurymartiny/reacher-microservices/commit/c1cb9c26bba7ab660258bd3d21d09cf446da0246))

## 0.1.0 (2020-05-02)


### Features

* Add /verify/demo endpoint ([5b036b2](https://github.com/reacherhq/backend/commit/5b036b2b2fc7d9fa1740dbb1a29b07ec78e3153f))
* Add a Dockerfile with Tor ([#24](https://github.com/reacherhq/backend/issues/24)) ([53210fc](https://github.com/amaurymartiny/reacher-microservices/commit/53210fcca03d1f4b6baad7573b18c49432e389e7))
* Add bulk verification ([46a418e](https://github.com/reacherhq/backend/commit/46a418e40f9ff1e896eaa00ddeba9d3d6da9abac))
* Add find or create user ([f06e96a](https://github.com/reacherhq/backend/commit/f06e96a117dcd2eb0535e70e18e69e15d144d48b))
* Add HTTP server inside Dockerfile ([#28](https://github.com/reacherhq/backend/issues/28)) ([f82610e](https://github.com/amaurymartiny/reacher-microservices/commit/f82610ecdc6f360e3be8f076fab793b13fb88251))
* Add serverless rust for email-exists ([3b186fe](https://github.com/reacherhq/backend/commit/3b186fee406af38faf6ec5e82cb68f0d30599b55))


### Bug Fixes

* Allow usage of express middlewares ([a37b8f5](https://github.com/reacherhq/backend/commit/a37b8f5bcbeea366f4c658f6dad4becf38245eeb))
* Return HTTP error when verification fails ([#30](https://github.com/reacherhq/backend/issues/30)) ([9074768](https://github.com/amaurymartiny/reacher-microservices/commit/90747689ff83640aa5b8b37d54a2f0b09cc433b3))
