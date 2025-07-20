# Changelog

## [0.4.0](https://github.com/andrewferrier/memy/compare/v0.3.0...v0.4.0) (2025-07-20)


### Features

* Add RELEASE_PAT token ([1b3ef3d](https://github.com/andrewferrier/memy/commit/1b3ef3de862e5851515806440ae4e5868f522067))


### Bug Fixes

* Dummy ([b315971](https://github.com/andrewferrier/memy/commit/b315971034a08f71c593691130df3f83519e4d80))

## [0.3.0](https://github.com/andrewferrier/memy/compare/v0.2.0...v0.3.0) (2025-07-20)


### Features

* Add missing-files-on-note-warn config param ([be8c38a](https://github.com/andrewferrier/memy/commit/be8c38ac298fa65cfff14f635b5750a0db1e68a9))
* Rename denylist_silent to denylist ([fb0177e](https://github.com/andrewferrier/memy/commit/fb0177ef8a1331ae5c63a3caad9936a21d1d52f1))
* Support generate-config &lt;filename&gt; - closes [#19](https://github.com/andrewferrier/memy/issues/19) ([932f8b4](https://github.com/andrewferrier/memy/commit/932f8b419d6274de85e76fa317ffc1ec5767b84c))

## [0.2.0](https://github.com/andrewferrier/memy/compare/v0.1.0...v0.2.0) (2025-07-18)


### Features

* --list, --note now subcommands - closes [#4](https://github.com/andrewferrier/memy/issues/4) ([b809a0f](https://github.com/andrewferrier/memy/commit/b809a0fa1ea5ebb534fa8c6a5395ded869ff7e71))
* ~/.local/state rather than ~/.cache for DB - closes [#8](https://github.com/andrewferrier/memy/issues/8) ([cc0da56](https://github.com/andrewferrier/memy/commit/cc0da56bc259077a40be85c206da917353ced0fa))
* Add --verbose and --debug flags - closes [#16](https://github.com/andrewferrier/memy/issues/16) ([d3c7140](https://github.com/andrewferrier/memy/commit/d3c7140e1710bba1d64f6e79b9cd44ac4549b065))
* Add generate-config command ([f4411ad](https://github.com/andrewferrier/memy/commit/f4411ada9082ad1a923c769ca59a61cc64348780))
* Add git version in --version ([b9849f5](https://github.com/andrewferrier/memy/commit/b9849f5fd69106aa9b81ca06221e106aba387f9b))
* Add support for outputting shell completions ([e664cf4](https://github.com/andrewferrier/memy/commit/e664cf498f52e81597339a5ce319f2b0e8aee64b))
* Implement logic to check database version - closes [#13](https://github.com/andrewferrier/memy/issues/13) ([9d47fc9](https://github.com/andrewferrier/memy/commit/9d47fc9fda004b310baa07ba67d23b625ad33afb))
* Implement path denylist - closes [#1](https://github.com/andrewferrier/memy/issues/1) ([428b347](https://github.com/andrewferrier/memy/commit/428b3476ea999d423a60b180080c7eddf7390ac4))
* normalize_symlinks_on_note option - closes [#12](https://github.com/andrewferrier/memy/issues/12) ([9ac62e3](https://github.com/andrewferrier/memy/commit/9ac62e39f1a9aa5f7fd90b30d394227e8b42991d))
* Print out db_path in debug output ([f287837](https://github.com/andrewferrier/memy/commit/f287837643ebf2f9f060208be93e41ef88cc2caa))


### Bug Fixes

* False +ve on dead code ([909d59a](https://github.com/andrewferrier/memy/commit/909d59a0da312d9e412a5a9f7b35ec8a265d614d))
* Use integer timestamps, not text ([912a2ac](https://github.com/andrewferrier/memy/commit/912a2aca725575cfe7e9ad1d391e44f39c08b0f8))

## 0.1.0 (2025-07-15)


### Features

* Add --files/--dirs only flags ([3aa2f83](https://github.com/andrewferrier/memy/commit/3aa2f83d8a162926b26ce4c9f09d23575267d63a))
* Add option to include frecency score ([046f152](https://github.com/andrewferrier/memy/commit/046f15290fe3a98a7dc4dae84400e6862d392938))
* Add some logging ([30c84f5](https://github.com/andrewferrier/memy/commit/30c84f55f4db984632adfc032daf9ee37ecb9eb1))
* First version ([e4ffe44](https://github.com/andrewferrier/memy/commit/e4ffe441ccd7db34ef9bc275cb7a876203a3ab4e))
* Remove recency bias as an option ([e2ae174](https://github.com/andrewferrier/memy/commit/e2ae174cd51a714bcc6851d2d82f556536b58299))


### Bug Fixes

* Remove files from DB once deleted ([504d5c7](https://github.com/andrewferrier/memy/commit/504d5c70905e8ee371ea222760dec9b4875d1db1))
