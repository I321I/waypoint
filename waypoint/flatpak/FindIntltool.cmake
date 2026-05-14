# Minimal FindIntltool.cmake for libayatana-common build inside flatpak.
#
# 用途：libayatana-common 的 po/CMakeLists.txt 呼叫
#   find_package(Intltool REQUIRED)
#   intltool_install_translations(ALL GETTEXT_PACKAGE ${GETTEXT_PACKAGE})
# 正常由 Debian cmake-extras 提供 FindIntltool.cmake，flatpak runtime 沒有。
#
# 我們在 flatpak build 不打包翻譯（Waypoint 本身不依賴 libayatana-common 翻譯），
# 把 intltool_install_translations 做成 no-op 跳過。

set(Intltool_FOUND TRUE)

function(intltool_install_translations)
    # no-op
endfunction()
