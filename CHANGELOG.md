# Changelog

## [0.11.0](https://github.com/andrewferrier/memy/compare/v0.10.0...v0.11.0) (2025-09-02)


### Features

* Add man pages to Debian package - closes [#60](https://github.com/andrewferrier/memy/issues/60) ([b0e7f6a](https://github.com/andrewferrier/memy/commit/b0e7f6ab0ab2b07ac471a42befb4b5ecb829e691))


### Bug Fixes

* Include full subcommand in man pages ([f986ca9](https://github.com/andrewferrier/memy/commit/f986ca9e4ad8e90ec003deb4a973a0ebfe2cca59))
* Make expand_tilde more robust ([c490234](https://github.com/andrewferrier/memy/commit/c490234c123ac09373f9cc07de4c7cab1acbd3c5))


### Performance Improvements

* Use transaction for note_path to speed up ([a470725](https://github.com/andrewferrier/memy/commit/a470725ef20531206bd70ea4ad8c866877fb510a))
* Use unstable sort ([97fe8b2](https://github.com/andrewferrier/memy/commit/97fe8b2c67ac322c21dfc7c8212b7fcb1c9150a1))

## [0.10.0](https://github.com/andrewferrier/memy/compare/v0.9.0...v0.10.0) (2025-08-31)


### Features

* Add MemyFZF command to NeoVim hook ([53ab064](https://github.com/andrewferrier/memy/commit/53ab064c46c9b706287e30e87a11268a15063c27))
* Add MemyMiniPick to NeoVim hook ([dca5442](https://github.com/andrewferrier/memy/commit/dca54427db69741b42ce033bad44709491d0a6b7))
* Provide switch dir/file command for lf - closes [#49](https://github.com/andrewferrier/memy/issues/49) ([51112dd](https://github.com/andrewferrier/memy/commit/51112dd678cc65194ad66043040191ab9276ff64))

## [0.9.0](https://github.com/andrewferrier/memy/compare/v0.8.0...v0.9.0) (2025-08-30)


### Features

* 'default', not 'template' ([1ddbab5](https://github.com/andrewferrier/memy/commit/1ddbab5c2ecad64a41b508735c660c771d00bc11))
* Add CSV output support ([e18bab8](https://github.com/andrewferrier/memy/commit/e18bab8b04ecfcee5ecbba271f1b1c927d28d854))
* Add fish and ranger hooks, and improve instructions ([9a0f3ab](https://github.com/andrewferrier/memy/commit/9a0f3ab084b42c1c5a4dfc514388e28f9b9ffe16))
* Add tracing support - closes [#45](https://github.com/andrewferrier/memy/issues/45) ([226ac4d](https://github.com/andrewferrier/memy/commit/226ac4d61a2eaa11540d58a10a63e05f48569b2a))
* JSON output - closes [#44](https://github.com/andrewferrier/memy/issues/44) ([7863d8b](https://github.com/andrewferrier/memy/commit/7863d8bfde908f4ff284736bf1134a07e64de641))


### Bug Fixes

* Change Debian build logic to build consistent name ([4ea802c](https://github.com/andrewferrier/memy/commit/4ea802c577629e1bca3a7a647534ef478fe96945))
* Remove use of unwrap() ([2e0a777](https://github.com/andrewferrier/memy/commit/2e0a777bb968290e771fd1ea3b8f887a008eb635))
* Replace atty crate ([4966216](https://github.com/andrewferrier/memy/commit/496621685856c6b95675a830d33437c56b7a22bc))
* Sort list of hooks - closes [#48](https://github.com/andrewferrier/memy/issues/48) ([1ddc24c](https://github.com/andrewferrier/memy/commit/1ddc24c6ee1dddbb9cd2d5d6b142072bd9170af9))

## [0.8.0](https://github.com/andrewferrier/memy/compare/v0.7.0...v0.8.0) (2025-08-15)


### Features

* Add more debugging output ([2321559](https://github.com/andrewferrier/memy/commit/2321559dd599dedcfda6c0566d9c60ac618d62df))
* Add NeoVim hook ([a984f02](https://github.com/andrewferrier/memy/commit/a984f0271b5be51045accaa165f3f927335ae52a))
* Add Vim hook ([e9ed9db](https://github.com/andrewferrier/memy/commit/e9ed9db07b4c8ff4639cc35184efb3345f59e882))


### Bug Fixes

* Add missing function param ([4563ed7](https://github.com/andrewferrier/memy/commit/4563ed783a1349065ba9cb147de6d66f77ba8223))
* Don't create memy XDG path if doesn't exist ([6ece3bc](https://github.com/andrewferrier/memy/commit/6ece3bceaabe1639e51e8879a0d0b8c561bb9d18))

## [0.7.0](https://github.com/andrewferrier/memy/compare/v0.6.0...v0.7.0) (2025-08-12)


### Features

* Additional debugging for CLI ([4f61374](https://github.com/andrewferrier/memy/commit/4f61374d5305a402ccfa631c421981e681d2171c))
* Additional debugging for config ([5eeeb3d](https://github.com/andrewferrier/memy/commit/5eeeb3d9434ff7563334380c177218a244eda887))
* Implement --config - closes [#31](https://github.com/andrewferrier/memy/issues/31) ([301485b](https://github.com/andrewferrier/memy/commit/301485b47fee6401a418595b05f5d8ebda1a3555))
* Improve option description ([98ac18c](https://github.com/andrewferrier/memy/commit/98ac18c3fd8f9521a885157a5cd67343e4d4f424))
* Warn if no paths are passed in to note ([401ba2d](https://github.com/andrewferrier/memy/commit/401ba2d7b53d12244971256521a5643b9d56f2c4))


### Bug Fixes

* Add denied files ignore for bash too ([5d509d0](https://github.com/andrewferrier/memy/commit/5d509d0f333b36b35fcceaaea58b4348c56eceac))
* Default PROMPT_COMMAND if not set - closes [#37](https://github.com/andrewferrier/memy/issues/37) ([a520adb](https://github.com/andrewferrier/memy/commit/a520adbd1d093911774fe67342554c81693426af))
* Error on incorrect fields in config ([0885c8e](https://github.com/andrewferrier/memy/commit/0885c8e0651c9c546ffa54fd29ed7f03e515605c))
* Warns when navigating to denied dirs - closes [#36](https://github.com/andrewferrier/memy/issues/36) ([30388e5](https://github.com/andrewferrier/memy/commit/30388e56fcae80977e3ca4dcd623733d86fc56d6))

## [0.6.0](https://github.com/andrewferrier/memy/compare/v0.5.0...v0.6.0) (2025-08-06)


### Features

* Add bash hook ([4575255](https://github.com/andrewferrier/memy/commit/457525515da14ef60a404b0dca28288ebcdcd8a4))
* Add initial zsh hook ([0acf076](https://github.com/andrewferrier/memy/commit/0acf07623d199aac8adda11e49198864aa3c0eeb))
* Add plugin capability ([5fb10a8](https://github.com/andrewferrier/memy/commit/5fb10a865a9e46bff205daff37d24b3bd7452607))
* Build man pages - closes [#27](https://github.com/andrewferrier/memy/issues/27) ([319c404](https://github.com/andrewferrier/memy/commit/319c4046e86dfda4ab3ca1f9f47253cb0b6bc9a2))
* New frecency algorithm & recency_bias option - closes [#29](https://github.com/andrewferrier/memy/issues/29) ([76d57ed](https://github.com/andrewferrier/memy/commit/76d57ed634c85d15e98029a3ba631b8363607b50))
* Produce config file annotated with comments - closes [#28](https://github.com/andrewferrier/memy/issues/28) ([4af4154](https://github.com/andrewferrier/memy/commit/4af4154a7c67c0f2b5170bf52b4dac13a87f53f5))
* Remove --include-frecency-score option ([1ec6d07](https://github.com/andrewferrier/memy/commit/1ec6d07acabded6fb0728c29d0b66fa6a74d6d67))


### Bug Fixes

* Expand tilde when noting - closes [#32](https://github.com/andrewferrier/memy/issues/32) ([4f71a24](https://github.com/andrewferrier/memy/commit/4f71a2470fde9b5e964bcca3dec74ad740666130))
* Sort list of hooks ([1f4fe4f](https://github.com/andrewferrier/memy/commit/1f4fe4fc901294c0e91a28c66b80d408702b472c))


### Performance Improvements

* Reduce size of dependencies ([fb28641](https://github.com/andrewferrier/memy/commit/fb286412b27c8675a2b0757b34ad79ea170b016e))

## [0.5.0](https://github.com/andrewferrier/memy/compare/v0.4.0...v0.5.0) (2025-07-27)


### Features

* Add denied_files_on_list config for 'list' command ([3d30097](https://github.com/andrewferrier/memy/commit/3d30097fb8e6f4820719cc94ba3dc98751995a07))
* gitignore pattern, add denied_files_warn_on_note ([087e142](https://github.com/andrewferrier/memy/commit/087e14272ae678f2599f9bbde702acd9de63a179))
* More aggressively prefer recent files ([643f8d5](https://github.com/andrewferrier/memy/commit/643f8d5b80c84207e8d7bd9be8c4ec3b72d8d887))
* Rename to missing_files_warn_on_note ([1b13ad0](https://github.com/andrewferrier/memy/commit/1b13ad0a5818d4bb7b06909d500b311a2289a53f))


### Bug Fixes

* Add aider* to gitignore ([c76fe77](https://github.com/andrewferrier/memy/commit/c76fe778daa718bb903432c9178e6b2847794c00))


### Performance Improvements

* Cache config load - closes [#22](https://github.com/andrewferrier/memy/issues/22) ([70fd476](https://github.com/andrewferrier/memy/commit/70fd4760774b7ecd03870408094aae779af6e36f))
* Reduce dependencies ([4566568](https://github.com/andrewferrier/memy/commit/45665686ade26059a97db82d1caf62fc0b17377e))
* Use metadata to detect file existence ([f21ead5](https://github.com/andrewferrier/memy/commit/f21ead5f5920500c7af04fb7aa4a471d1d86d2ce))

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
