.PHONY: all dirs artifacts_dir contracts_dir contracts

artifacts_dir := artifacts
contracts_dir := contracts
library_dir := library
contracts := $(patsubst $(contracts_dir)/%.sol,%.sol,$(wildcard $(contracts_dir)/*.sol))

all: dirs $(patsubst %.sol,$(artifacts_dir)/%.bin,$(contracts))

dirs: | $(artifacts_dir) $(library_dir)

deploy: $(patsubst %.sol,$(artifacts_dir)/%.bin,$(contracts))
	./deploy.sh

$(artifacts_dir)/%.bin: $(contracts_dir)/%.sol
	solc --overwrite --abi --bin --base-path . --include-path library -o artifacts $<

$(artifacts_dir):
	@mkdir -p $(artifacts_dir)

$(library_dir): $(library_dir)/v2-core-1.0.1
	@mkdir -p $(library_dir)
	wget https://github.com/OpenZeppelin/openzeppelin-contracts/archive/refs/tags/v5.3.0.tar.gz
	tar xCf $(library_dir) v5.3.0.tar.gz
	rm v5.3.0.tar.gz

$(library_dir)/v2-core-1.0.1: 
	wget https://github.com/Uniswap/v2-core/archive/refs/tags/v1.0.1.tar.gz
	tar xCf $(library_dir) v1.0.1.tar.gz
	rm v1.0.1.tar.gz
	./solc-0.5.16 --overwrite --abi --bin -o artifacts library/v2-core-1.0.1/contracts/UniswapV2Pair.sol
	./solc-0.5.16 --overwrite --abi --bin -o artifacts library/v2-core-1.0.1/contracts/UniswapV2Factory.sol

