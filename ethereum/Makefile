.PHONY: all dirs artifacts_dir contracts_dir contracts

artifacts_dir := artifacts
contracts_dir := contracts
library_dir := library
contracts := $(patsubst $(contracts_dir)/%.sol,%.sol,$(wildcard $(contracts_dir)/*.sol))

all: dirs $(patsubst %.sol,$(artifacts_dir)/%.bin,$(contracts))

dirs: | $(artifacts_dir) $(library_dir) $(library_dir)/v2-core-1.0.1 $(library_dir)/openzeppelin-contracts-5.3.0

format:
	prettier $(contracts_dir)

swab: $(artifacts_dir)/UniSwab.bin
	eth contract:deploy -n hardhat --pk hat2 artifacts/UniSwab.bin | jq .address

deploy: $(patsubst %.sol,$(artifacts_dir)/%.bin,$(contracts))
	./deploy.sh

$(artifacts_dir)/%.bin: $(contracts_dir)/%.sol
	solc --overwrite --abi --bin --base-path . --include-path library -o artifacts $<

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
	mkdir bin
	wget --quiet -O bin/solc-0.5.16 https://github.com/ethereum/solc-bin/raw/refs/heads/gh-pages/linux-amd64/solc-linux-amd64-v0.5.16+commit.9c3226ce
	chmod 755 ./bin/solc-0.5.16
	./bin/solc-0.5.16 --overwrite --abi --bin -o artifacts library/v2-core-1.0.1/contracts/UniswapV2Pair.sol
	./bin/solc-0.5.16 --overwrite --abi --bin -o artifacts library/v2-core-1.0.1/contracts/UniswapV2Factory.sol

