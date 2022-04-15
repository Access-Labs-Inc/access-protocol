package utils

import (
	"context"

	"encoding/binary"

	"os"

	"github.com/portto/solana-go-sdk/client"
	"github.com/portto/solana-go-sdk/common"
	"github.com/portto/solana-go-sdk/rpc"
)

const OFFSET_STAKE = 1 + 32
const OFFSET_MIN_CREATION = 1 + 32 + 8 + 32 + 8
const OFFSET_MIN_CURRENT = 1 + 1 + 2 + 4 


// Verifies that a user has enough tokens staked
func CheckStake(owner string) bool {
	c := client.NewClient(rpc.DevnetRPCEndpoint)

	
	seeds := [][]byte{	
		[]byte("stake_account"), 
		common.PublicKeyFromString(owner).Bytes(), 
		common.PublicKeyFromString(os.Getenv("STAKE_POOL_KEY")).Bytes(),
	}
	key, _, err := common.FindProgramAddress(seeds, common.PublicKeyFromString(os.Getenv("STAKE_POOL_KEY")))

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
	min := uint64(binary.LittleEndian.Uint64(info.Data[OFFSET_MIN_CREATION: OFFSET_MIN_CREATION + 8]))


	info, err = c.GetAccountInfo(context.TODO(), os.Getenv("STAKE_POOL_KEY"))

	if err != nil {
		// Stake pool not found
		return false
	}

	pool_minimum := uint64(binary.LittleEndian.Uint64(info.Data[OFFSET_MIN_CURRENT: OFFSET_MIN_CURRENT + 8]))

	return stake > min || stake > pool_minimum
}