.PHONY: all dirs artifacts_dir contracts_dir contracts
artifacts_dir := artifacts
contracts_dir := contracts
library_dir := library
contracts := $(patsubst $(contracts_dir)/%.sol,%.sol,$(wildcard $(contracts_dir)/*.sol))

all: dirs $(patsubst %.sol,$(artifacts_dir)/%.bin,$(contracts))

dirs: | $(artifacts_dir) $(library_dir)

deploy:
	eth contract:deploy -n hardhat --pk hat1  ./$(artifacts_dir)/UsdCoin.bin

$(artifacts_dir)/%.bin: $(contracts_dir)/%.sol
	solc --overwrite --abi --bin --base-path . --include-path library -o artifacts $<

$(artifacts_dir):
	@mkdir -p $(artifacts_dir)

$(library_dir): 
	@mkdir -p $(library_dir)
	wget https://github.com/OpenZeppelin/openzeppelin-contracts/archive/refs/tags/v5.3.0.tar.gz
	tar xCf $(library_dir) v5.3.0.tar.gz
	rm v5.3.0.tar.gz

