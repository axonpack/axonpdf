Yoga 3.2.1
Source: https://github.com/facebook/yoga, tag v3.2.1, commit 042f5013152eb81c1552dec945b88f7b95ca350f
The latest Yoga release, the same as yoga-layout 3.2.1 on npm.
Copied unchanged: the yoga/ directory and LICENSE. The inner yoga/ name has to stay: Yoga includes
its own headers as <yoga/...>, so this folder is the include path. Another version goes in a sibling
folder (vendor/yoga/<version>/), and YOGA_VERSION in crates/engine/build.rs picks the one to build.

This folder is on the C++ include path. Never add a file named like a standard header here
(for example VERSION): macOS file systems ignore case, so it would shadow <version>.
