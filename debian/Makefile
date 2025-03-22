PACKAGE_NAME=tinyghettobox
VERSION=1.0.0
ARCH=arm64
TARGET_DIR=../target/aarch64-unknown-linux-gnu
BIN_SOURCE_DIR=$(TARGET_DIR)/release
OUTPUT_DIR=$(TARGET_DIR)/debian
INSTALL_DIR=$(OUTPUT_DIR)/srv/tinyghettobox
SYSTEMD_DIR=$(OUTPUT_DIR)/etc/systemd/system
SPOTIFYD_CONF_DIR=$(OUTPUT_DIR)/etc/spotifyd

all: debian_package

debian_package: clean create_dirs copy_binaries copy_webui copy_config copy_debian_config
	dpkg-deb -Zxz --build $(OUTPUT_DIR) ./debian.deb
	mv ./debian.deb $(OUTPUT_DIR)/$(PACKAGE_NAME)_$(VERSION)_$(ARCH).deb

create_dirs:
	mkdir -p $(OUTPUT_DIR)/DEBIAN
	mkdir -p $(INSTALL_DIR)
	mkdir -p $(SYSTEMD_DIR)
	mkdir -p $(SPOTIFYD_CONF_DIR)
	mkdir -p $(OUTPUT_DIR)/srv/tinyghettobox/admin_interface_ui

copy_binaries:
	cp $(BIN_SOURCE_DIR)/admin_interface_server $(INSTALL_DIR)
	cp $(BIN_SOURCE_DIR)/user_interface $(INSTALL_DIR)
	cp $(BIN_SOURCE_DIR)/spotifyd $(INSTALL_DIR)

copy_webui:
	cp -r ../admin_interface/web_ui/dist/* $(OUTPUT_DIR)/srv/tinyghettobox/admin_interface_ui

copy_config:
	cp ./etc/systemd/system/tgb.user-interface.service $(SYSTEMD_DIR)
	cp ./etc/systemd/system/tgb.admin-interface.service $(SYSTEMD_DIR)
	cp ./etc/systemd/system/spotifyd.service $(SYSTEMD_DIR)
	cp ./etc/spotifyd/spotifyd.conf $(SPOTIFYD_CONF_DIR)

copy_debian_config:
	cp -r ./DEBIAN/* $(OUTPUT_DIR)/DEBIAN/
	chmod 0755 $(OUTPUT_DIR)/DEBIAN/postinst

clean:
	rm -rf $(OUTPUT_DIR)