from typing import List
from borsh_construct import U8, String, CStruct
from solana.transaction import TransactionInstruction, AccountMeta
from solana.publickey import PublicKey


class CloseStakePoolInstruction:
    schema = CStruct(
        "tag" / U8,
    )

    def serialize(self,
                  ) -> str:
        return self.schema.build({
            "tag": 10,
        })

    def getInstruction(self,
                       programId: PublicKey,
                       stake_pool_account: PublicKey,
                       owner: PublicKey,
                       ) -> TransactionInstruction:
        data = self.serialize(
        )
        keys: List[AccountMeta] = []
        keys.append(AccountMeta(stake_pool_account,
                                False, True))
        keys.append(AccountMeta(owner,
                                True, True))
        return TransactionInstruction(keys, programId, data)


class ChangeInflationInstruction:
    schema = CStruct(
        "tag" / U8,
        "daily_inflation" / U64,
    )

    def serialize(self,
                  daily_inflation: int,
                  ) -> str:
        return self.schema.build({
            "tag": 12,
            "daily_inflation": daily_inflation,
        })

    def getInstruction(self,
                       programId: PublicKey,
                       central_state: PublicKey,
                       authority: PublicKey,
                       daily_inflation: int,
                       ) -> TransactionInstruction:
        data = self.serialize(
            daily_inflation,
        )
        keys: List[AccountMeta] = []
        keys.append(AccountMeta(central_state,
                                False, True))
        keys.append(AccountMeta(authority,
                                True, False))
        return TransactionInstruction(keys, programId, data)


class UnlockBondTokensInstruction:
    schema = CStruct(
        "tag" / U8,
    )

    def serialize(self,
                  ) -> str:
        return self.schema.build({
            "tag": 15,
        })

    def getInstruction(self,
                       programId: PublicKey,
                       bond_account: PublicKey,
                       bond_owner: PublicKey,
                       mint: PublicKey,
                       access_token_destination: PublicKey,
                       central_state: PublicKey,
                       stake_pool: PublicKey,
                       pool_vault: PublicKey,
                       spl_token_program: PublicKey,
                       ) -> TransactionInstruction:
        data = self.serialize(
        )
        keys: List[AccountMeta] = []
        keys.append(AccountMeta(bond_account,
                                False, True))
        keys.append(AccountMeta(bond_owner,
                                True, False))
        keys.append(AccountMeta(mint,
                                False, False))
        keys.append(AccountMeta(access_token_destination,
                                False, True))
        keys.append(AccountMeta(central_state,
                                False, True))
        keys.append(AccountMeta(stake_pool,
                                False, True))
        keys.append(AccountMeta(pool_vault,
                                False, True))
        keys.append(AccountMeta(spl_token_program,
                                False, False))
        return TransactionInstruction(keys, programId, data)


class AdminFreezeInstruction:
    schema = CStruct(
        "tag" / U8,
    )

    def serialize(self,
                  ) -> str:
        return self.schema.build({
            "tag": 20,
        })

    def getInstruction(self,
                       programId: PublicKey,
                       authority: PublicKey,
                       account_to_freeze: PublicKey,
                       central_state: PublicKey,
                       ) -> TransactionInstruction:
        data = self.serialize(
        )
        keys: List[AccountMeta] = []
        keys.append(AccountMeta(authority,
                                True, False))
        keys.append(AccountMeta(account_to_freeze,
                                False, True))
        keys.append(AccountMeta(central_state,
                                False, False))
        return TransactionInstruction(keys, programId, data)


class ChangePoolMinimumInstruction:
    schema = CStruct(
        "tag" / U8,
        "new_minimum" / U64,
    )

    def serialize(self,
                  new_minimum: int,
                  ) -> str:
        return self.schema.build({
            "tag": 18,
            "new_minimum": new_minimum,
        })

    def getInstruction(self,
                       programId: PublicKey,
                       stake_pool: PublicKey,
                       stake_pool_owner: PublicKey,
                       new_minimum: int,
                       ) -> TransactionInstruction:
        data = self.serialize(
            new_minimum,
        )
        keys: List[AccountMeta] = []
        keys.append(AccountMeta(stake_pool,
                                False, True))
        keys.append(AccountMeta(stake_pool_owner,
                                True, False))
        return TransactionInstruction(keys, programId, data)


class ClaimPoolRewardsInstruction:
    schema = CStruct(
        "tag" / U8,
    )

    def serialize(self,
                  ) -> str:
        return self.schema.build({
            "tag": 7,
        })

    def getInstruction(self,
                       programId: PublicKey,
                       stake_pool: PublicKey,
                       owner: PublicKey,
                       rewards_destination: PublicKey,
                       central_state: PublicKey,
                       mint: PublicKey,
                       spl_token_program: PublicKey,
                       ) -> TransactionInstruction:
        data = self.serialize(
        )
        keys: List[AccountMeta] = []
        keys.append(AccountMeta(stake_pool,
                                False, True))
        keys.append(AccountMeta(owner,
                                True, False))
        keys.append(AccountMeta(rewards_destination,
                                False, True))
        keys.append(AccountMeta(central_state,
                                False, False))
        keys.append(AccountMeta(mint,
                                False, True))
        keys.append(AccountMeta(spl_token_program,
                                False, False))
        return TransactionInstruction(keys, programId, data)


class SignBondInstruction:
    schema = CStruct(
        "tag" / U8,
        "seller_index" / U64,
    )

    def serialize(self,
                  seller_index: int,
                  ) -> str:
        return self.schema.build({
            "tag": 14,
            "seller_index": seller_index,
        })

    def getInstruction(self,
                       programId: PublicKey,
                       seller: PublicKey,
                       bond_account: PublicKey,
                       seller_index: int,
                       ) -> TransactionInstruction:
        data = self.serialize(
            seller_index,
        )
        keys: List[AccountMeta] = []
        keys.append(AccountMeta(seller,
                                True, False))
        keys.append(AccountMeta(bond_account,
                                False, True))
        return TransactionInstruction(keys, programId, data)


class ClaimRewardsInstruction:
    schema = CStruct(
        "tag" / U8,
    )

    def serialize(self,
                  ) -> str:
        return self.schema.build({
            "tag": 8,
        })

    def getInstruction(self,
                       programId: PublicKey,
                       stake_pool: PublicKey,
                       stake_account: PublicKey,
                       owner: PublicKey,
                       rewards_destination: PublicKey,
                       central_state: PublicKey,
                       mint: PublicKey,
                       spl_token_program: PublicKey,
                       ) -> TransactionInstruction:
        data = self.serialize(
        )
        keys: List[AccountMeta] = []
        keys.append(AccountMeta(stake_pool,
                                False, True))
        keys.append(AccountMeta(stake_account,
                                False, True))
        keys.append(AccountMeta(owner,
                                True, False))
        keys.append(AccountMeta(rewards_destination,
                                False, True))
        keys.append(AccountMeta(central_state,
                                False, False))
        keys.append(AccountMeta(mint,
                                False, True))
        keys.append(AccountMeta(spl_token_program,
                                False, False))
        return TransactionInstruction(keys, programId, data)


class StakeInstruction:
    schema = CStruct(
        "tag" / U8,
        "amount" / U64,
    )

    def serialize(self,
                  amount: int,
                  ) -> str:
        return self.schema.build({
            "tag": 4,
            "amount": amount,
        })

    def getInstruction(self,
                       programId: PublicKey,
                       central_state_account: PublicKey,
                       stake_account: PublicKey,
                       stake_pool: PublicKey,
                       owner: PublicKey,
                       source_token: PublicKey,
                       spl_token_program: PublicKey,
                       vault: PublicKey,
                       amount: int,
                       ) -> TransactionInstruction:
        data = self.serialize(
            amount,
        )
        keys: List[AccountMeta] = []
        keys.append(AccountMeta(central_state_account,
                                False, True))
        keys.append(AccountMeta(stake_account,
                                False, True))
        keys.append(AccountMeta(stake_pool,
                                False, True))
        keys.append(AccountMeta(owner,
                                True, False))
        keys.append(AccountMeta(source_token,
                                False, True))
        keys.append(AccountMeta(spl_token_program,
                                False, False))
        keys.append(AccountMeta(vault,
                                False, True))
        return TransactionInstruction(keys, programId, data)


class CreateCentralStateInstruction:
    schema = CStruct(
        "tag" / U8,
        "daily_inflation" / U64,
        "authority" / U8[32],
    )

    def serialize(self,
                  daily_inflation: int,
                  authority: PublicKey,
                  ) -> str:
        return self.schema.build({
            "tag": 0,
            "daily_inflation": daily_inflation,
            "authority": authority,
        })

    def getInstruction(self,
                       programId: PublicKey,
                       state_account: PublicKey,
                       system_program: PublicKey,
                       fee_payer: PublicKey,
                       mint: PublicKey,
                       daily_inflation: int,
                       authority: PublicKey,
                       ) -> TransactionInstruction:
        data = self.serialize(
            daily_inflation,
            authority,
        )
        keys: List[AccountMeta] = []
        keys.append(AccountMeta(state_account,
                                False, True))
        keys.append(AccountMeta(system_program,
                                False, False))
        keys.append(AccountMeta(fee_payer,
                                True, True))
        keys.append(AccountMeta(mint,
                                False, False))
        return TransactionInstruction(keys, programId, data)


class ChangePoolMultiplierInstruction:
    schema = CStruct(
        "tag" / U8,
        "new_multiplier" / U64,
    )

    def serialize(self,
                  new_multiplier: int,
                  ) -> str:
        return self.schema.build({
            "tag": 21,
            "new_multiplier": new_multiplier,
        })

    def getInstruction(self,
                       programId: PublicKey,
                       stake_pool: PublicKey,
                       stake_pool_owner: PublicKey,
                       new_multiplier: int,
                       ) -> TransactionInstruction:
        data = self.serialize(
            new_multiplier,
        )
        keys: List[AccountMeta] = []
        keys.append(AccountMeta(stake_pool,
                                False, True))
        keys.append(AccountMeta(stake_pool_owner,
                                True, False))
        return TransactionInstruction(keys, programId, data)


class CloseStakeAccountInstruction:
    schema = CStruct(
        "tag" / U8,
    )

    def serialize(self,
                  ) -> str:
        return self.schema.build({
            "tag": 11,
        })

    def getInstruction(self,
                       programId: PublicKey,
                       stake_account: PublicKey,
                       owner: PublicKey,
                       ) -> TransactionInstruction:
        data = self.serialize(
        )
        keys: List[AccountMeta] = []
        keys.append(AccountMeta(stake_account,
                                False, True))
        keys.append(AccountMeta(owner,
                                True, True))
        return TransactionInstruction(keys, programId, data)


class CrankInstruction:
    schema = CStruct(
        "tag" / U8,
    )

    def serialize(self,
                  ) -> str:
        return self.schema.build({
            "tag": 9,
        })

    def getInstruction(self,
                       programId: PublicKey,
                       stake_pool: PublicKey,
                       central_state: PublicKey,
                       ) -> TransactionInstruction:
        data = self.serialize(
        )
        keys: List[AccountMeta] = []
        keys.append(AccountMeta(stake_pool,
                                False, True))
        keys.append(AccountMeta(central_state,
                                False, False))
        return TransactionInstruction(keys, programId, data)


class ExecuteUnstakeInstruction:
    schema = CStruct(
        "tag" / U8,
    )

    def serialize(self,
                  ) -> str:
        return self.schema.build({
            "tag": 6,
        })

    def getInstruction(self,
                       programId: PublicKey,
                       stake_account: PublicKey,
                       stake_pool: PublicKey,
                       owner: PublicKey,
                       destination_token: PublicKey,
                       spl_token_program: PublicKey,
                       vault: PublicKey,
                       ) -> TransactionInstruction:
        data = self.serialize(
        )
        keys: List[AccountMeta] = []
        keys.append(AccountMeta(stake_account,
                                False, True))
        keys.append(AccountMeta(stake_pool,
                                False, True))
        keys.append(AccountMeta(owner,
                                True, False))
        keys.append(AccountMeta(destination_token,
                                False, True))
        keys.append(AccountMeta(spl_token_program,
                                False, False))
        keys.append(AccountMeta(vault,
                                False, True))
        return TransactionInstruction(keys, programId, data)


class ClaimBondInstruction:
    schema = CStruct(
        "tag" / U8,
    )

    def serialize(self,
                  ) -> str:
        return self.schema.build({
            "tag": 16,
        })

    def getInstruction(self,
                       programId: PublicKey,
                       bond_account: PublicKey,
                       buyer: PublicKey,
                       quote_token_source: PublicKey,
                       quote_token_destination: PublicKey,
                       stake_pool: PublicKey,
                       access_mint: PublicKey,
                       pool_vault: PublicKey,
                       central_state: PublicKey,
                       spl_token_program: PublicKey,
                       ) -> TransactionInstruction:
        data = self.serialize(
        )
        keys: List[AccountMeta] = []
        keys.append(AccountMeta(bond_account,
                                False, True))
        keys.append(AccountMeta(buyer,
                                True, False))
        keys.append(AccountMeta(quote_token_source,
                                False, True))
        keys.append(AccountMeta(quote_token_destination,
                                False, True))
        keys.append(AccountMeta(stake_pool,
                                False, True))
        keys.append(AccountMeta(access_mint,
                                False, True))
        keys.append(AccountMeta(pool_vault,
                                False, True))
        keys.append(AccountMeta(central_state,
                                False, True))
        keys.append(AccountMeta(spl_token_program,
                                False, False))
        return TransactionInstruction(keys, programId, data)


class CreateStakeAccountInstruction:
    schema = CStruct(
        "tag" / U8,
        "nonce" / U8,
        "owner" / U8[32],
    )

    def serialize(self,
                  nonce: int,
                  owner: PublicKey,
                  ) -> str:
        return self.schema.build({
            "tag": 3,
            "nonce": nonce,
            "owner": owner,
        })

    def getInstruction(self,
                       programId: PublicKey,
                       stake_account: PublicKey,
                       system_program: PublicKey,
                       stake_pool: PublicKey,
                       fee_payer: PublicKey,
                       nonce: int,
                       owner: PublicKey,
                       ) -> TransactionInstruction:
        data = self.serialize(
            nonce,
            owner,
        )
        keys: List[AccountMeta] = []
        keys.append(AccountMeta(stake_account,
                                False, True))
        keys.append(AccountMeta(system_program,
                                False, False))
        keys.append(AccountMeta(stake_pool,
                                False, False))
        keys.append(AccountMeta(fee_payer,
                                True, True))
        return TransactionInstruction(keys, programId, data)


class ClaimBondRewardsInstruction:
    schema = CStruct(
        "tag" / U8,
    )

    def serialize(self,
                  ) -> str:
        return self.schema.build({
            "tag": 17,
        })

    def getInstruction(self,
                       programId: PublicKey,
                       stake_pool: PublicKey,
                       bond_account: PublicKey,
                       bond_owner: PublicKey,
                       rewards_destination: PublicKey,
                       central_state: PublicKey,
                       mint: PublicKey,
                       spl_token_program: PublicKey,
                       ) -> TransactionInstruction:
        data = self.serialize(
        )
        keys: List[AccountMeta] = []
        keys.append(AccountMeta(stake_pool,
                                False, True))
        keys.append(AccountMeta(bond_account,
                                False, True))
        keys.append(AccountMeta(bond_owner,
                                True, False))
        keys.append(AccountMeta(rewards_destination,
                                False, True))
        keys.append(AccountMeta(central_state,
                                False, False))
        keys.append(AccountMeta(mint,
                                False, True))
        keys.append(AccountMeta(spl_token_program,
                                False, False))
        return TransactionInstruction(keys, programId, data)


class CreateBondInstruction:
    schema = CStruct(
        "tag" / U8,
        "buyer" / U8[32],
        "total_amount_sold" / U64,
        "total_quote_amount" / U64,
        "quote_mint" / U8[32],
        "seller_token_account" / U8[32],
        "unlock_start_date" / I64,
        "unlock_period" / I64,
        "unlock_amount" / U64,
        "seller_index" / U64,
    )

    def serialize(self,
                  buyer: PublicKey,
                  total_amount_sold: int,
                  total_quote_amount: int,
                  quote_mint: PublicKey,
                  seller_token_account: PublicKey,
                  unlock_start_date: int,
                  unlock_period: int,
                  unlock_amount: int,
                  seller_index: int,
                  ) -> str:
        return self.schema.build({
            "tag": 13,
            "buyer": buyer,
            "total_amount_sold": total_amount_sold,
            "total_quote_amount": total_quote_amount,
            "quote_mint": quote_mint,
            "seller_token_account": seller_token_account,
            "unlock_start_date": unlock_start_date,
            "unlock_period": unlock_period,
            "unlock_amount": unlock_amount,
            "seller_index": seller_index,
        })

    def getInstruction(self,
                       programId: PublicKey,
                       seller: PublicKey,
                       bond_account: PublicKey,
                       stake_pool: PublicKey,
                       system_program: PublicKey,
                       fee_payer: PublicKey,
                       buyer: PublicKey,
                       total_amount_sold: int,
                       total_quote_amount: int,
                       quote_mint: PublicKey,
                       seller_token_account: PublicKey,
                       unlock_start_date: int,
                       unlock_period: int,
                       unlock_amount: int,
                       seller_index: int,
                       ) -> TransactionInstruction:
        data = self.serialize(
            buyer,
            total_amount_sold,
            total_quote_amount,
            quote_mint,
            seller_token_account,
            unlock_start_date,
            unlock_period,
            unlock_amount,
            seller_index,
        )
        keys: List[AccountMeta] = []
        keys.append(AccountMeta(seller,
                                True, True))
        keys.append(AccountMeta(bond_account,
                                False, True))
        keys.append(AccountMeta(stake_pool,
                                False, False))
        keys.append(AccountMeta(system_program,
                                False, False))
        keys.append(AccountMeta(fee_payer,
                                True, True))
        return TransactionInstruction(keys, programId, data)


class UnstakeInstruction:
    schema = CStruct(
        "tag" / U8,
        "amount" / U64,
    )

    def serialize(self,
                  amount: int,
                  ) -> str:
        return self.schema.build({
            "tag": 5,
            "amount": amount,
        })

    def getInstruction(self,
                       programId: PublicKey,
                       central_state_account: PublicKey,
                       stake_account: PublicKey,
                       stake_pool: PublicKey,
                       owner: PublicKey,
                       amount: int,
                       ) -> TransactionInstruction:
        data = self.serialize(
            amount,
        )
        keys: List[AccountMeta] = []
        keys.append(AccountMeta(central_state_account,
                                False, True))
        keys.append(AccountMeta(stake_account,
                                False, True))
        keys.append(AccountMeta(stake_pool,
                                False, True))
        keys.append(AccountMeta(owner,
                                True, False))
        return TransactionInstruction(keys, programId, data)


class CreateStakePoolInstruction:
    schema = CStruct(
        "tag" / U8,
        "owner" / U8[32],
        "minimum_stake_amount" / U64,
    )

    def serialize(self,
                  owner: PublicKey,
                  minimum_stake_amount: int,
                  ) -> str:
        return self.schema.build({
            "tag": 1,
            "owner": owner,
            "minimum_stake_amount": minimum_stake_amount,
        })

    def getInstruction(self,
                       programId: PublicKey,
                       stake_pool_account: PublicKey,
                       system_program: PublicKey,
                       fee_payer: PublicKey,
                       vault: PublicKey,
                       owner: PublicKey,
                       minimum_stake_amount: int,
                       ) -> TransactionInstruction:
        data = self.serialize(
            owner,
            minimum_stake_amount,
        )
        keys: List[AccountMeta] = []
        keys.append(AccountMeta(stake_pool_account,
                                False, True))
        keys.append(AccountMeta(system_program,
                                False, False))
        keys.append(AccountMeta(fee_payer,
                                True, True))
        keys.append(AccountMeta(vault,
                                False, False))
        return TransactionInstruction(keys, programId, data)


class AdminMintInstruction:
    schema = CStruct(
        "tag" / U8,
        "amount" / U64,
    )

    def serialize(self,
                  amount: int,
                  ) -> str:
        return self.schema.build({
            "tag": 19,
            "amount": amount,
        })

    def getInstruction(self,
                       programId: PublicKey,
                       authority: PublicKey,
                       mint: PublicKey,
                       access_token_destination: PublicKey,
                       central_state: PublicKey,
                       spl_token_program: PublicKey,
                       amount: int,
                       ) -> TransactionInstruction:
        data = self.serialize(
            amount,
        )
        keys: List[AccountMeta] = []
        keys.append(AccountMeta(authority,
                                True, False))
        keys.append(AccountMeta(mint,
                                False, True))
        keys.append(AccountMeta(access_token_destination,
                                False, True))
        keys.append(AccountMeta(central_state,
                                False, False))
        keys.append(AccountMeta(spl_token_program,
                                False, False))
        return TransactionInstruction(keys, programId, data)


class ActivateStakePoolInstruction:
    schema = CStruct(
        "tag" / U8,
    )

    def serialize(self,
                  ) -> str:
        return self.schema.build({
            "tag": 2,
        })

    def getInstruction(self,
                       programId: PublicKey,
                       authority: PublicKey,
                       stake_pool: PublicKey,
                       central_state: PublicKey,
                       ) -> TransactionInstruction:
        data = self.serialize(
        )
        keys: List[AccountMeta] = []
        keys.append(AccountMeta(authority,
                                True, False))
        keys.append(AccountMeta(stake_pool,
                                False, True))
        keys.append(AccountMeta(central_state,
                                False, False))
        return TransactionInstruction(keys, programId, data)
