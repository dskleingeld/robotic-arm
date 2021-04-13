#!/usr/bin/env bash
set -e

UDEV_FILE=/etc/udev/rules.d/70-st-link.rules
UDEV_RULE1='# STM32F3DISCOVERY rev A/B - ST-LINK/V2'
UDEV_RULE2='ATTRS{idVendor}=="0483", ATTRS{idProduct}=="3748", TAG+="uaccess"'
UDEV_RULE3='# STM32F3DISCOVERY rev C+ - ST-LINK/V2-1'
UDEV_RULE4='ATTRS{idVendor}=="0483", ATTRS{idProduct}=="374b", TAG+="uaccess"'

function install_tools {
	echo "installing tools"
	# rustup override set nightly-thumbv7em-none-eabi
	rustup override set nightly
	rustup target add thumbv7em-none-eabihf

	cargo install cargo-binutils probe-run
	rustup component add llvm-tools-preview

	sudo apt install gdb-multiarch openocd
}

# add udev rules if they do not yet exist
function fix_udev_rules {
	if [ ! -f "$UDEV_FILE" ]; then
		sudo sh -c "echo \"${UDEV_RULE1}\" >> ${UDEV_FILE}"
		sudo sh -c "echo \"${UDEV_RULE2}\" >> ${UDEV_FILE}"
		sudo sh -c "echo \"${UDEV_RULE3}\" >> ${UDEV_FILE}"
		sudo sh -c "echo \"${UDEV_RULE4}\" >> ${UDEV_FILE}"
		sudo udevadm control --reload-rules
		echo "created udev rules: $UDEV_FILE"
	fi

}

install_tools
fix_udev_rules
