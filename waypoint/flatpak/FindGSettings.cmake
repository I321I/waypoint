# Minimal FindGSettings.cmake for libayatana-common build inside flatpak.
#
# 用途：libayatana-common 的 data/CMakeLists.txt 呼叫
#   find_package(GSettings REQUIRED)
#   add_schema("org.ayatana.common.gschema.xml")
# 但 libayatana-common 沒附 FindGSettings.cmake，正常情況下由 Debian
# cmake-extras package 提供。flatpak runtime（org.gnome.Platform/46）也沒有，
# ECM 也沒附（v6.0.0 確認），所以這裡用最小實作補上 add_schema() macro。
#
# 行為與標準 add_schema() 一致：把 *.gschema.xml 安裝到 share/glib-2.0/schemas。

set(GSettings_FOUND TRUE)

function(add_schema SCHEMA_FILE)
    install(
        FILES "${CMAKE_CURRENT_SOURCE_DIR}/${SCHEMA_FILE}"
        DESTINATION "${CMAKE_INSTALL_DATAROOTDIR}/glib-2.0/schemas"
    )
endfunction()
