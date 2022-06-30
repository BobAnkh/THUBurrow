# CHANGELOG

## Unreleased

Changes unreleased.

### Documentation

- readme:
  - update docs website ([b60efc8](https://github.com/BobAnkh/THUBurrow/commit/b60efc819c20d1a8627378de5690b2c09870672d)) ([#335](https://github.com/BobAnkh/THUBurrow/pull/335))
  - update contributors ([0c5d43e](https://github.com/BobAnkh/THUBurrow/commit/0c5d43e08af43dfa89515a952962f8a7bc8e3041))

## [v1.0.0](https://github.com/BobAnkh/THUBurrow/releases/tag/v1.0.0) - 2021-12-29 16:01:41

The first formal release version of the project. Add all the fundemantal and necessary functions.

### Feature

- addattention:
  - add attention in burrow page (#110) ([0b38db4](https://github.com/BobAnkh/THUBurrow/commit/0b38db495f59c18e47d548a421dac42332047ec4)) ([#110](https://github.com/BobAnkh/THUBurrow/pull/110))

- health:
  - add undefined interface (#109) ([bbf30ec](https://github.com/BobAnkh/THUBurrow/commit/bbf30ecbd4a5a6c0147b7ac11de574f67b4b77cd)) ([#109](https://github.com/BobAnkh/THUBurrow/pull/109))
  - add health check ([0d9aad3](https://github.com/BobAnkh/THUBurrow/commit/0d9aad31beaf8674fa69dd8b422b39a5438b4950)) ([#60](https://github.com/BobAnkh/THUBurrow/pull/60))

- upload:
  - add upload image to post (#97) ([5dcbca7](https://github.com/BobAnkh/THUBurrow/commit/5dcbca77394561747fc8a2e37ba581a52b62ed1d)) ([#97](https://github.com/BobAnkh/THUBurrow/pull/97))
  - add upload image ([adbf0de](https://github.com/BobAnkh/THUBurrow/commit/adbf0de9042dc80abbdca07fc3c84c21a27836ed)) ([#84](https://github.com/BobAnkh/THUBurrow/pull/84))

- profile:
  - add change password, fix api bug of my burrow (#90) ([1442f2f](https://github.com/BobAnkh/THUBurrow/commit/1442f2fa24a5ac123e3efcbcf36b0c8b8e12e764)) ([#90](https://github.com/BobAnkh/THUBurrow/pull/90))
  - support add and modify burrow on profile page (#66) ([92f79ac](https://github.com/BobAnkh/THUBurrow/commit/92f79acbbcff12bdbc07b895baff2588011ac017)) ([#66](https://github.com/BobAnkh/THUBurrow/pull/66))
  - add a vanilla user profile page (#39) ([0319686](https://github.com/BobAnkh/THUBurrow/commit/0319686f206cc8342fcf69b49d3e8c80b180bdec)) ([#39](https://github.com/BobAnkh/THUBurrow/pull/39))

- storage:
  - limit interfaces to admin use ([a11d8eb](https://github.com/BobAnkh/THUBurrow/commit/a11d8eb6a60298e64136bfc9cc409479b37c51e5)) ([#60](https://github.com/BobAnkh/THUBurrow/pull/60))
  - add storage table ([e2304df](https://github.com/BobAnkh/THUBurrow/commit/e2304df5509c2dd4f8df1d6f55fd1e3bcefe7d74)) ([#60](https://github.com/BobAnkh/THUBurrow/pull/60))
  - add tables for storage (#17) ([54605c1](https://github.com/BobAnkh/THUBurrow/commit/54605c17aca7eeea75b62a1f26975042683f390b)) ([#60](https://github.com/BobAnkh/THUBurrow/pull/60))
  - add referrer check for download ([d46ac3e](https://github.com/BobAnkh/THUBurrow/commit/d46ac3e94cd860e95916db49da61c12f70226240)) ([#60](https://github.com/BobAnkh/THUBurrow/pull/60))

- post:
  - add update and delete post (#78) ([26ea0e8](https://github.com/BobAnkh/THUBurrow/commit/26ea0e82eeef4c47865ec29102ba4936dd804e1a)) ([#78](https://github.com/BobAnkh/THUBurrow/pull/78))
  - add function of modify reply (#58) ([a1612fd](https://github.com/BobAnkh/THUBurrow/commit/a1612fdda5c6f3c329b3164a2ae02de245123b40)) ([#58](https://github.com/BobAnkh/THUBurrow/pull/58))
  - add like and favorites in post list and post details page  (#28) ([a07f8f6](https://github.com/BobAnkh/THUBurrow/commit/a07f8f6a5ddc0c1c51523eebaacbdb634a030ef4)) ([#28](https://github.com/BobAnkh/THUBurrow/pull/28))
  - add view post detail and reply  (#21) ([15b22f2](https://github.com/BobAnkh/THUBurrow/commit/15b22f2f9c573cc0fecca64125eed2d78854943d)) ([#21](https://github.com/BobAnkh/THUBurrow/pull/21))

- admin:
  - add routes for admin operation ([8454bfa](https://github.com/BobAnkh/THUBurrow/commit/8454bfac992ccd22b23ffeef6884e15570c0ec07)) ([#60](https://github.com/BobAnkh/THUBurrow/pull/60))
  - add db table for admin ([1175d5c](https://github.com/BobAnkh/THUBurrow/commit/1175d5c4574362f01c514b069424c1c7a8918500)) ([#60](https://github.com/BobAnkh/THUBurrow/pull/60))

- config:
  - improve config extraction and test ([4aa99f2](https://github.com/BobAnkh/THUBurrow/commit/4aa99f23a2e7c93d73c63f118eecd6fe5cfabbb0)) ([#60](https://github.com/BobAnkh/THUBurrow/pull/60))

- section:
  - update the section (#70) ([d07417d](https://github.com/BobAnkh/THUBurrow/commit/d07417d26d79b7864566779579eedf733f48ba71)) ([#70](https://github.com/BobAnkh/THUBurrow/pull/70))

- cors:
  - add cors for catch-all ([94678e4](https://github.com/BobAnkh/THUBurrow/commit/94678e408bb89dd6481576b24ea310c1cfe1d9b7)) ([#60](https://github.com/BobAnkh/THUBurrow/pull/60))
  - add support for cors (#1) ([e1d3302](https://github.com/BobAnkh/THUBurrow/commit/e1d3302ec260dbe418613908ec91d8a77c596984)) ([#60](https://github.com/BobAnkh/THUBurrow/pull/60))

- user:
  - add logout (#67) ([aed85e1](https://github.com/BobAnkh/THUBurrow/commit/aed85e1a1a7d754bfe1218a3e7f95f674c78788e)) ([#67](https://github.com/BobAnkh/THUBurrow/pull/67))
  - retrieve-password (#59) ([e19cb73](https://github.com/BobAnkh/THUBurrow/commit/e19cb73f6668a1532aa4b917e51a069286bd23cd)) ([#59](https://github.com/BobAnkh/THUBurrow/pull/59))

- logout:
  - add interface for logout ([6035746](https://github.com/BobAnkh/THUBurrow/commit/60357461e3b175c5a2040f031e7907b92b6883c4)) ([#60](https://github.com/BobAnkh/THUBurrow/pull/60))

- burrow:
  - add abandoned page and fix several bugs (#53) ([b9f2c12](https://github.com/BobAnkh/THUBurrow/commit/b9f2c128bbc77b1f514ae1cd687d82c80f5d7dfc)) ([#53](https://github.com/BobAnkh/THUBurrow/pull/53))
  - implement core concept burrow (#20) ([145d96f](https://github.com/BobAnkh/THUBurrow/commit/145d96f8a61630edeb8e4b8353659ec8f99d278d)) ([#60](https://github.com/BobAnkh/THUBurrow/pull/60))

- create:
  - add create page  (#52) ([ae78bbf](https://github.com/BobAnkh/THUBurrow/commit/ae78bbfb10813e71ec21bb74024a55647869fe9c)) ([#52](https://github.com/BobAnkh/THUBurrow/pull/52))

- frontend,component:
  - add markdown component ([873a546](https://github.com/BobAnkh/THUBurrow/commit/873a546558ee5a16d66b5e8af556f08d14d4f6fe)) ([#44](https://github.com/BobAnkh/THUBurrow/pull/44))

- page:
  - modify the page (#40) ([892b36d](https://github.com/BobAnkh/THUBurrow/commit/892b36dd87332c956715c566cbe763fe283fd668)) ([#40](https://github.com/BobAnkh/THUBurrow/pull/40))

- revise regist:
  - revise register.tsx and update the burrow folder (#36) ([3d8ae17](https://github.com/BobAnkh/THUBurrow/commit/3d8ae1759a8e060762063fadfd4f610f17ca5c62)) ([#36](https://github.com/BobAnkh/THUBurrow/pull/36))

- search:
  - modify the search page, removed the StandardFormRow component, postList component adds a tag  (#29) ([626e773](https://github.com/BobAnkh/THUBurrow/commit/626e773fa7d96b2ea2b528d0aa04c55926c451d7)) ([#29](https://github.com/BobAnkh/THUBurrow/pull/29))
  - add search burrows and posts(#26) ([cad677f](https://github.com/BobAnkh/THUBurrow/commit/cad677fb85e73f1f2d2eef7bfc2637cbecb6eb22)) ([#26](https://github.com/BobAnkh/THUBurrow/pull/26))

- burrowpage:
  - add the function to view the details of the burrow ([7837ec8](https://github.com/BobAnkh/THUBurrow/commit/7837ec852981f8a2468cb6ec857f47c1d6f3a227)) ([#27](https://github.com/BobAnkh/THUBurrow/pull/27))

- general:
  - feature(email): extract email checker (#18) ([5402f8c](https://github.com/BobAnkh/THUBurrow/commit/5402f8c60ccfc36bf437f4488f48d8a24274ab2d)) ([#60](https://github.com/BobAnkh/THUBurrow/pull/60))
  - feature(storage): add image storage (#9) ([6c501db](https://github.com/BobAnkh/THUBurrow/commit/6c501db7ba31be18274201de164ebf6586fce323)) ([#60](https://github.com/BobAnkh/THUBurrow/pull/60))

- postlist:
  - show the list of posts in home page (#12) ([3f78335](https://github.com/BobAnkh/THUBurrow/commit/3f78335efc803074a16b1d6098525bf8d10e2f85)) ([#12](https://github.com/BobAnkh/THUBurrow/pull/12))

- login:
  - complete user login, register (#11) ([e140b47](https://github.com/BobAnkh/THUBurrow/commit/e140b47ba1ebd01da6d6666da525391de0413da1)) ([#11](https://github.com/BobAnkh/THUBurrow/pull/11))

- auth:
  - add user signup, login and authentication (#8) ([fdb7f4b](https://github.com/BobAnkh/THUBurrow/commit/fdb7f4b4d457488ef0c04c36e95112c25a34f81b)) ([#60](https://github.com/BobAnkh/THUBurrow/pull/60))

- frontend:
  - initialize a frontend project (#4) ([e15f6ea](https://github.com/BobAnkh/THUBurrow/commit/e15f6eadf735ab55072d173c21070efc7dc74a06)) ([#4](https://github.com/BobAnkh/THUBurrow/pull/4))

- redis:
  - add support for cache database (#3) ([bf3f4aa](https://github.com/BobAnkh/THUBurrow/commit/bf3f4aa684302d135a7c330448791b8a1d0642f3)) ([#60](https://github.com/BobAnkh/THUBurrow/pull/60))

- id:
  - add random unique id generator (#2) ([e3972dc](https://github.com/BobAnkh/THUBurrow/commit/e3972dc582b6eb620d23eb60e931e2552c48d8ba)) ([#60](https://github.com/BobAnkh/THUBurrow/pull/60))

- backend:
  - add backend sample ([2337d3a](https://github.com/BobAnkh/THUBurrow/commit/2337d3ad86c5b3356ae0adaa4ca78040cfffc0ee)) ([#118](https://github.com/BobAnkh/THUBurrow/pull/118))

### Bug Fixes

- attention:
  - add ! to fix the bug (#125) ([f67e8fd](https://github.com/BobAnkh/THUBurrow/commit/f67e8fdb4d34e1c94d118ca7716f7866d25b74cb)) ([#125](https://github.com/BobAnkh/THUBurrow/pull/125))

- trending:
  - trending (#124) ([08d6177](https://github.com/BobAnkh/THUBurrow/commit/08d61770692639780fcba3feee8b616681d1ac62)) ([#124](https://github.com/BobAnkh/THUBurrow/pull/124))
  - fix profile, add comment button (#119) ([9e54632](https://github.com/BobAnkh/THUBurrow/commit/9e546322bef73bb63661028f080f6bebb75ac5f4)) ([#119](https://github.com/BobAnkh/THUBurrow/pull/119))
  - fix trending, verification code, change password (#106) ([3fd5501](https://github.com/BobAnkh/THUBurrow/commit/3fd55012a219ac7fa3e559d2ed8b3a28339fe339)) ([#106](https://github.com/BobAnkh/THUBurrow/pull/106))

- burrow:
  - click good (#120) ([4ca3ac1](https://github.com/BobAnkh/THUBurrow/commit/4ca3ac1c03e7455c2485c9ddac73fd53e96f6163)) ([#120](https://github.com/BobAnkh/THUBurrow/pull/120))
  - change title and description for default burrow (#92) ([e099a8b](https://github.com/BobAnkh/THUBurrow/commit/e099a8b2bfed9b8b104b8c0e0d1224fcaa00026e)) ([#92](https://github.com/BobAnkh/THUBurrow/pull/92))

- desert:
  - check the users if they are the host (#117) ([a395b27](https://github.com/BobAnkh/THUBurrow/commit/a395b27cd969b844ca0622b033c67073e6d11bc0)) ([#117](https://github.com/BobAnkh/THUBurrow/pull/117))

- picture:
  - change picture and reply  (#116) ([dd2f1fe](https://github.com/BobAnkh/THUBurrow/commit/dd2f1febdc7af76bbdb65e77f0ff272ad085bb4d)) ([#116](https://github.com/BobAnkh/THUBurrow/pull/116))
  - fix picture(#107) ([75d7a25](https://github.com/BobAnkh/THUBurrow/commit/75d7a2597060545135c414fe96c60b4e3bbffce5)) ([#107](https://github.com/BobAnkh/THUBurrow/pull/107))

- search:
  - contend have html language (#111) ([18697bb](https://github.com/BobAnkh/THUBurrow/commit/18697bb0777755b99df53146e8651e51d15488cc)) ([#111](https://github.com/BobAnkh/THUBurrow/pull/111))
  - remove {} in url (#94) ([246cea3](https://github.com/BobAnkh/THUBurrow/commit/246cea33497d45e64e2e812ed3797e5e06090607)) ([#94](https://github.com/BobAnkh/THUBurrow/pull/94))
  - update url,add show reply number (#91) ([176311e](https://github.com/BobAnkh/THUBurrow/commit/176311ecb9004405aa06ebfc18a9bcfa6b260ec7)) ([#91](https://github.com/BobAnkh/THUBurrow/pull/91))
  - change keyword to keywords,deactivateLike to DeactivateLike (#80) ([7250872](https://github.com/BobAnkh/THUBurrow/commit/7250872344846c4778a6edbdf9284ca19042f50c)) ([#80](https://github.com/BobAnkh/THUBurrow/pull/80))
  - delete the search id method, directly jump to the corresponding url (#73) ([6348079](https://github.com/BobAnkh/THUBurrow/commit/63480796ada726f8d168f5909d04434fea53abdc)) ([#73](https://github.com/BobAnkh/THUBurrow/pull/73))
  - split the search page into several components and add the ability to view posts by section   (#54) ([30e7f3f](https://github.com/BobAnkh/THUBurrow/commit/30e7f3f4b8b7cf1b12fd82e29c5aa725df52e161)) ([#54](https://github.com/BobAnkh/THUBurrow/pull/54))
  - rewrite the search page according to the interface (#49) ([5d9e5d5](https://github.com/BobAnkh/THUBurrow/commit/5d9e5d57c530fb711efc229a641f4fc8180a2dad)) ([#49](https://github.com/BobAnkh/THUBurrow/pull/49))

- post:
  - add post-id (#108) ([b443446](https://github.com/BobAnkh/THUBurrow/commit/b44344653ed443ac05be75383a81a30b14458afc)) ([#108](https://github.com/BobAnkh/THUBurrow/pull/108))
  - update api (#98) ([35864d9](https://github.com/BobAnkh/THUBurrow/commit/35864d9ef8a7e92141afd2cd4c7a58a19d031647)) ([#98](https://github.com/BobAnkh/THUBurrow/pull/98))
  - modify display like and collection logic (#87) ([56c6b6e](https://github.com/BobAnkh/THUBurrow/commit/56c6b6e87342f3ce7e194229ecaee4382858d3fc)) ([#87](https://github.com/BobAnkh/THUBurrow/pull/87))
  - update url (#79) ([058080f](https://github.com/BobAnkh/THUBurrow/commit/058080fd94291f526842c855bf85fc587be9ced8)) ([#79](https://github.com/BobAnkh/THUBurrow/pull/79))
  - update the url (#69) ([902cf9e](https://github.com/BobAnkh/THUBurrow/commit/902cf9e23d26cd8f7fa144712dd117a013427092)) ([#69](https://github.com/BobAnkh/THUBurrow/pull/69))

- postid:
  - show postid (#105) ([879eb73](https://github.com/BobAnkh/THUBurrow/commit/879eb73d61d4986bef34040827e9e1866c2f84b6)) ([#105](https://github.com/BobAnkh/THUBurrow/pull/105))

- delete:
  - improved markdown support (#103) ([f080ab4](https://github.com/BobAnkh/THUBurrow/commit/f080ab46693dfc254fc9316acc85b4a407cf6906)) ([#103](https://github.com/BobAnkh/THUBurrow/pull/103))
  - fix delete (#102) ([8ec92db](https://github.com/BobAnkh/THUBurrow/commit/8ec92db173c52e8d29afe912c23178e98d429b52)) ([#102](https://github.com/BobAnkh/THUBurrow/pull/102))

- upload:
  - fix upload (#101) ([5ae99fb](https://github.com/BobAnkh/THUBurrow/commit/5ae99fb9102b90427b61c3ef7b1b6cd575ffb175)) ([#101](https://github.com/BobAnkh/THUBurrow/pull/101))
  - change setState (#99) ([82eb659](https://github.com/BobAnkh/THUBurrow/commit/82eb65956a5e1f3dc1196a023ed6ce8eb2a28d0b)) ([#99](https://github.com/BobAnkh/THUBurrow/pull/99))
  - update the url(#85) ([b9575bb](https://github.com/BobAnkh/THUBurrow/commit/b9575bb6399c42e4097429dfb3a7ce344207c906)) ([#85](https://github.com/BobAnkh/THUBurrow/pull/85))

- profile:
  - fix profile (#100) ([172ff54](https://github.com/BobAnkh/THUBurrow/commit/172ff54a0bc2d14c78d2df96830132d167af197b)) ([#100](https://github.com/BobAnkh/THUBurrow/pull/100))
  -  fix profile, trending, header(#93) ([7306115](https://github.com/BobAnkh/THUBurrow/commit/73061159101a48f49a254d2bdced07b2f501cf09)) ([#93](https://github.com/BobAnkh/THUBurrow/pull/93))

- section:
  - set limit on selecting tag and section (#95) ([2279eae](https://github.com/BobAnkh/THUBurrow/commit/2279eae06ef1ef4f14f9d2842a162f1c72d032db)) ([#95](https://github.com/BobAnkh/THUBurrow/pull/95))

- email:
  - add new email suffix and revise the font (#86) ([c3747aa](https://github.com/BobAnkh/THUBurrow/commit/c3747aa1bd077515be02f225a1735ee42a0bbbb6)) ([#86](https://github.com/BobAnkh/THUBurrow/pull/86))

- register:
  - fix the error of verifying password and sending email verification code (#71) ([e58b5c9](https://github.com/BobAnkh/THUBurrow/commit/e58b5c990e4fb6ec85a461f4fd6ea12bddd25de0)) ([#71](https://github.com/BobAnkh/THUBurrow/pull/71))
  - encrypt password (#61) ([7f5f65f](https://github.com/BobAnkh/THUBurrow/commit/7f5f65fc52e250f935f1cbe0b9fa07236948e401)) ([#61](https://github.com/BobAnkh/THUBurrow/pull/61))
  - change url (#46) ([ce5f889](https://github.com/BobAnkh/THUBurrow/commit/ce5f8896ced147bebb650cb1ff884d4413f421b4)) ([#46](https://github.com/BobAnkh/THUBurrow/pull/46))

- burrowpage:
  - fix bugs and make the function of counting down work correctly (#50) ([4e0a67b](https://github.com/BobAnkh/THUBurrow/commit/4e0a67b637df2e0a854ea30d29ee2b7474d1d65e)) ([#50](https://github.com/BobAnkh/THUBurrow/pull/50))

- fronetend:
  - modify the axios default configuration to support cross-domain requests in all pages (#31) ([35452db](https://github.com/BobAnkh/THUBurrow/commit/35452db5ee0a947b41f1bb1ccf6efaf8a3292c1d)) ([#31](https://github.com/BobAnkh/THUBurrow/pull/31))

- frontend:
  - modify the axios default configuration to support cross-domain requests (#30) ([32c4c1e](https://github.com/BobAnkh/THUBurrow/commit/32c4c1eb1fca32cf3a50d65538503a49c74b0bd9)) ([#30](https://github.com/BobAnkh/THUBurrow/pull/30))

- homepage:
  - change pre-render to render (#15) ([d308512](https://github.com/BobAnkh/THUBurrow/commit/d308512daf80b474da8e7563c450406091275dcb)) ([#15](https://github.com/BobAnkh/THUBurrow/pull/15))

### Documentation

- readme:
  - add codacy quality badge ([4ab6d8a](https://github.com/BobAnkh/THUBurrow/commit/4ab6d8a958bab2169967e3f743e8d42396468337)) ([#141](https://github.com/BobAnkh/THUBurrow/pull/141))
  - update contributors ([2fd666d](https://github.com/BobAnkh/THUBurrow/commit/2fd666d76494c4ee47630fb0ea494b80eced5aae))
  - add formula for hot search ([d6aff01](https://github.com/BobAnkh/THUBurrow/commit/d6aff01c16b8b2f0e9fe19fde85b94bd149b6fd3)) ([#60](https://github.com/BobAnkh/THUBurrow/pull/60))

- search:
  - add docs for search and storage (#76) ([77ff0ed](https://github.com/BobAnkh/THUBurrow/commit/77ff0ed9b1591427835b6aa30965fd88bf15d727)) ([#60](https://github.com/BobAnkh/THUBurrow/pull/60))

- user:
  - add comment, benchmark, test for user_logout (#75) ([f07dd5e](https://github.com/BobAnkh/THUBurrow/commit/f07dd5e369642f7e398833be35611053d45e8824)) ([#60](https://github.com/BobAnkh/THUBurrow/pull/60))

- content:
  - add docs for models/content (#74) ([5fe14d2](https://github.com/BobAnkh/THUBurrow/commit/5fe14d2504b2ffc6edbff4a8a3459a9fe1c238e2)) ([#60](https://github.com/BobAnkh/THUBurrow/pull/60))
  - add docs for content and burrow (#72) ([c01d4d9](https://github.com/BobAnkh/THUBurrow/commit/c01d4d938deff951e68db0274ffd0ab4f4d3d581)) ([#60](https://github.com/BobAnkh/THUBurrow/pull/60))

- backend:
  - add docs for trending & health ([92ac961](https://github.com/BobAnkh/THUBurrow/commit/92ac9619fdadee9708709cd3bbe9f4bd7ac652e8)) ([#60](https://github.com/BobAnkh/THUBurrow/pull/60))

- convention:
  - add coding conventions ([219a7e3](https://github.com/BobAnkh/THUBurrow/commit/219a7e3d9a5e02bfd32c78d8fbd58e1c2964f0da)) ([#60](https://github.com/BobAnkh/THUBurrow/pull/60))

### Refactor

- backend:
  - reorganize codes into workspace ([3dd40ec](https://github.com/BobAnkh/THUBurrow/commit/3dd40ecd8cbdf7a72c88db0d16b27c3d4071bd11)) ([#141](https://github.com/BobAnkh/THUBurrow/pull/141))
  - adjust structure ([73c3478](https://github.com/BobAnkh/THUBurrow/commit/73c3478c3469798975b978da7926a92eddd5d2fa)) ([#60](https://github.com/BobAnkh/THUBurrow/pull/60))
  - move init from utils to setup ([3dbfdbf](https://github.com/BobAnkh/THUBurrow/commit/3dbfdbf29be24331e176130e33c8aefaf8c1cb47)) ([#60](https://github.com/BobAnkh/THUBurrow/pull/60))
  - reorganize interfaces and file structure (#45) ([e28c366](https://github.com/BobAnkh/THUBurrow/commit/e28c36636ffbc8edfde4bf49c7a4ae66170e20ab)) ([#60](https://github.com/BobAnkh/THUBurrow/pull/60))
  - reorganize file structure ([6d3d957](https://github.com/BobAnkh/THUBurrow/commit/6d3d957b4af9a4668b183cc0142fdeeebb324609)) ([#60](https://github.com/BobAnkh/THUBurrow/pull/60))

- login:
  - reconsitution login and findback,delete and move some css (#33) ([f850706](https://github.com/BobAnkh/THUBurrow/commit/f85070636c12a8967cfbcafd13f2b2580b53878d)) ([#33](https://github.com/BobAnkh/THUBurrow/pull/33))

- lib:
  - reorganize file structure ([70e8f52](https://github.com/BobAnkh/THUBurrow/commit/70e8f52046bd736f112aed2e0e9643f79f42bf18)) ([#60](https://github.com/BobAnkh/THUBurrow/pull/60))

### Performance Improvements

- image:
  - improve build process ([7bda79c](https://github.com/BobAnkh/THUBurrow/commit/7bda79c47540777000bc812009a10695f712db7b)) ([#141](https://github.com/BobAnkh/THUBurrow/pull/141))

- log:
  - improve log output ([b0c2031](https://github.com/BobAnkh/THUBurrow/commit/b0c20310f295c4eeac584029fb8c617e8cdb462b)) ([#60](https://github.com/BobAnkh/THUBurrow/pull/60))

- auth:
  - improve sso, cookie and cors ([570e7f5](https://github.com/BobAnkh/THUBurrow/commit/570e7f506f51c033058581739852cabac8eb1182)) ([#60](https://github.com/BobAnkh/THUBurrow/pull/60))

\* *This CHANGELOG was automatically generated by [auto-generate-changelog](https://github.com/BobAnkh/auto-generate-changelog)*
