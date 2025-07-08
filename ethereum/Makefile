.PHONY: all dirs

artifacts_dir := artifacts
contracts_dir := contracts
library_dir := library
bin_dir := bin
contracts := $(patsubst $(contracts_dir)/%.sol,%.sol,$(wildcard $(contracts_dir)/*.sol))
arch := $(shell uname -m | sed 's/x86_64/amd64/;s/aarch64/arm64/')
$(info arch $(arch))
solc_bin_url := https://binaries.soliditylang.org/

all: dirs $(patsubst %.sol,$(artifacts_dir)/%.bin,$(contracts)) $(artifacts_dir)/WETH9.bin

dirs: | $(artifacts_dir) $(library_dir) $(library_dir)/v2-core-1.0.1 $(library_dir)/openzeppelin-contracts-5.3.0 $(bin_dir)/solc-0.5.16

format:
	prettier $(contracts_dir)

swab: $(artifacts_dir)/UniSwab.bin
	./deploy_swab.sh

deploy: $(patsubst %.sol,$(artifacts_dir)/%.bin,$(contracts))
	./deploy_uniswap.sh

$(artifacts_dir)/WETH9.bin:
	cp $(library_dir)/WETH9.bin $@

$(artifacts_dir)/%.bin: $(contracts_dir)/%.sol
	./bin/solc-latest --overwrite --abi --bin --base-path . --include-path library -o artifacts $<

$(artifacts_dir):
	@mkdir -p $(artifacts_dir)

$(library_dir):
	@mkdir -p $(library_dir)

$(library_dir)/openzeppelin-contracts-5.3.0:
	wget https://github.com/OpenZeppelin/openzeppelin-contracts/archive/refs/tags/v5.3.0.tar.gz
	tar xCf $(library_dir) v5.3.0.tar.gz
	rm v5.3.0.tar.gz

$(library_dir)/v2-core-1.0.1: 
	wget --quiet https://github.com/Uniswap/v2-core/archive/refs/tags/v1.0.1.tar.gz
	tar xCf $(library_dir) v1.0.1.tar.gz
	rm v1.0.1.tar.gz

$(bin_dir)/solc-0.5.16:
	mkdir -p bin
	wget --quiet -O bin/solc-0.5.16 $(solc_bin_url)linux-$(arch)/solc-linux-$(arch)-v0.5.16+commit.9c3226ce
	chmod 755 ./bin/solc-0.5.16
	./bin/solc-0.5.16 --overwrite --abi --bin -o artifacts library/v2-core-1.0.1/contracts/UniswapV2Pair.sol
	./bin/solc-0.5.16 --overwrite --abi --bin -o artifacts library/v2-core-1.0.1/contracts/UniswapV2Factory.sol

$(bin_dir)/solc-latest:
	wget --quiet -O bin/solc-latest $(solc_bin_url)linux-$(arch)/$(shell curl https://binaries.soliditylang.org/linux-$(arch)/list.json | jq '.releases[.latestRelease]')
	chmod 755 ./bin/solc-latest

