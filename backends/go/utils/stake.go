package utils

import (
	"context"

	"encoding/binary"

	"github.com/portto/solana-go-sdk/client"
	"github.com/portto/solana-go-sdk/common"
	"github.com/portto/solana-go-sdk/rpc"
)

const OFFSET = 1 + 32
const MINIMUM_STAKE uint64 = 1_000 // TODO CHANGE
const PROGRAM_ID = ""
const STAKE_POOL = ""


func CheckStake(owner string) bool {
	c := client.NewClient(rpc.DevnetRPCEndpoint)

	seeds := [][]byte{[]byte("stake_account"), []byte(owner), []byte(STAKE_POOL)}
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

	stake := uint64(binary.LittleEndian.Uint64(info.Data[OFFSET: OFFSET + 8]))

	return stake > MINIMUM_STAKE
}