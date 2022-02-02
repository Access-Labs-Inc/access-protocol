package utils

import (
	"context"
	"os"

	"encoding/binary"

	"github.com/portto/solana-go-sdk/client"
	"github.com/portto/solana-go-sdk/common"
	"github.com/portto/solana-go-sdk/rpc"

	"log"
)

const OFFSET_STAKE = 1 + 32
const OFFSET_MIN = 1 + 32 + 8 + 32 + 8
const PROGRAM_ID = "2ZsWiVGXzL4kgMDtSfeEJSV27fBnMptrdcNKKZygUoB8" // TODO Change
const STAKE_POOL = "Hs6emyaDnMSxJmGxnHhSmucJh1Q9jSysuKJ5yycWoUuC" // TODO Change

// Verifies that a user has enough tokens staked
func CheckStake(owner string) bool {
	c := client.NewClient(rpc.DevnetRPCEndpoint)
	log.SetOutput(os.Stdout)

	
	seeds := [][]byte{	
		[]byte("stake_account"), 
		common.PublicKeyFromString(owner).Bytes(), 
		common.PublicKeyFromString(STAKE_POOL).Bytes(),
	}
	key, _, err := common.FindProgramAddress(seeds, common.PublicKeyFromString(PROGRAM_ID))

	if err != nil {
		// Error deriving key
		return false
	}

	info, err := c.GetAccountInfo(context.TODO(), key.ToBase58())

	if err != nil {
		// Stake account not found
		return false
	}

	stake := uint64(binary.LittleEndian.Uint64(info.Data[OFFSET_STAKE: OFFSET_STAKE + 8]))
	min := uint64(binary.LittleEndian.Uint64(info.Data[OFFSET_MIN: OFFSET_MIN + 8]))


	return stake > min
}