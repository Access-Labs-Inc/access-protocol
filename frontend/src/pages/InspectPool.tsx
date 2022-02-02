import React, { useState } from "react";
import { useParams } from "react-router-dom";
import Card from "../components/Card";
import { usePoolInfo } from "../hooks/usePoolInfo";
import { styled } from "@mui/material/styles";
import { Button } from "@mui/material";
import {
  crank,
  stake,
  StakeAccount,
  CentralState,
  createStakeAccount,
} from "@access";
import { ACCESS_PROGRAM_ID } from "@access";
import { PublicKey } from "@solana/web3.js";
import { sendTx } from "../utils/send";
import { useWallet, useConnection } from "@solana/wallet-adapter-react";
import FormControl from "@mui/material/FormControl";
import InputLabel from "@mui/material/InputLabel";
import OutlinedInput from "@mui/material/OutlinedInput";
import {
  Token,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { useStakeAccount } from "../hooks/useStakeAccount";

const Title = styled("span")({
  fontSize: 25,
  fontWeight: "bold",
});

const Row = styled("div")({
  display: "flex",
  alignItems: "center",
  justifyContent: "space-between",
  width: "100%",
});

const Label = styled("span")({
  fontSize: 24,
  fontWeight: "bold",
});

const Value = styled("span")({
  fontSize: 24,
  opacity: 0.8,
});

const Container = styled("div")({
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  height: "100vh",
  flexDirection: "column",
});

const CardContainer = styled("div")({
  height: 300,
});

const InnerCard = styled("div")({
  padding: 20,
  width: 900,
  height: 550,
  display: "flex",
  justifyContent: "space-around",
  alignItems: "center",
  flexDirection: "column",
});

const FormControlStyled = styled(FormControl)({
  width: "90%",
  margin: 10,
});

const InspectPool = () => {
  const { key } = useParams();
  const [poolInfo] = usePoolInfo(key);
  const { connection } = useConnection();
  const { publicKey, sendTransaction } = useWallet();
  const [stakeAmount, setStakeAmount] = useState<null | string>(null);
  const [stakeAccount] = useStakeAccount(key);

  console.log(poolInfo);
  console.log(stakeAccount);

  const handleCrank = async () => {
    if (!key || !publicKey) return;
    try {
      const ix = await crank(new PublicKey(key), ACCESS_PROGRAM_ID);
      const tx = await sendTx(connection, publicKey, [ix], sendTransaction);
      console.log(tx);
    } catch (e) {
      console.log(e);
    }
  };

  const handleStake = async () => {
    if (!key || !publicKey || !stakeAmount) return;
    try {
      const [stakeKey] = await StakeAccount.getKey(
        ACCESS_PROGRAM_ID,
        publicKey,
        new PublicKey(key)
      );

      const stakeExists = !!(await connection.getAccountInfo(stakeKey))?.data;
      if (!stakeExists) {
        const create_ix = await createStakeAccount(
          new PublicKey(key),
          publicKey,
          publicKey,
          ACCESS_PROGRAM_ID
        );
        const tx = await sendTx(
          connection,
          publicKey,
          [create_ix],
          sendTransaction
        );
        console.log(tx);
      }

      // Using central state to get the ACCESS mint. Should be hardcoded in the NPM package once on devnet.
      const [centralKey] = await CentralState.getKey(ACCESS_PROGRAM_ID);
      const centralState = await CentralState.retrieve(connection, centralKey);

      const parsedStake = parseInt(stakeAmount); // Assumes decimals are given by the user

      // Assumes funds are held in an associated token account
      const source = await Token.getAssociatedTokenAddress(
        ASSOCIATED_TOKEN_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
        centralState.tokenMint,
        publicKey
      );
      const ix = await stake(
        connection,
        stakeKey,
        source,
        parsedStake,
        ACCESS_PROGRAM_ID
      );
      const tx = await sendTx(connection, publicKey, [ix], sendTransaction);
      console.log(tx);
    } catch (err) {
      console.log(err);
    }
  };

  return (
    <Container>
      <CardContainer>
        <Card>
          <InnerCard>
            <Title>Pool info</Title>
            <Row>
              <Label>Last claimed time</Label>
              <Value>{poolInfo?.lastClaimedTime.toNumber()}</Value>
            </Row>
            <Row>
              <Label>Last crank time</Label>
              <Value>{poolInfo?.lastCrankTime.toNumber()}</Value>
            </Row>
            <Row>
              <Label>Min stake amount</Label>
              <Value>{poolInfo?.minimumStakeAmount.toNumber()}</Value>
            </Row>
            <Row>
              <Label>Owner</Label>
              <Value>{poolInfo?.owner.toBase58()}</Value>
            </Row>
            <Row>
              <Label>Stakers multiplier (%)</Label>
              <Value>{poolInfo?.stakersMultiplier.toNumber()}</Value>
            </Row>
            <Row>
              <Label>Total staked</Label>
              <Value>{poolInfo?.totalStaked.toNumber()}</Value>
            </Row>{" "}
            <Row>
              <Label>Unstake period (s)</Label>
              <Value>{poolInfo?.unstakePeriod.toNumber()}</Value>
            </Row>
            <Button variant="contained" onClick={handleCrank}>
              Crank
            </Button>
            <Title>Stake</Title>
            <FormControlStyled>
              <InputLabel>Stake amount</InputLabel>
              <OutlinedInput
                type="text"
                id="component-outlined"
                value={stakeAmount}
                onChange={(e) => setStakeAmount(e.target.value.trim())}
                label="Stake amount"
              />
            </FormControlStyled>
            <Button variant="contained" onClick={handleStake}>
              Stake
            </Button>
            {stakeAccount && (
              <>
                <Row>
                  <Label>Last claimed time</Label>
                  <Value>{stakeAccount.lastClaimedTime.toNumber()}</Value>
                </Row>
                <Row>
                  <Label>Owner</Label>
                  <Value>{stakeAccount.owner.toBase58()}</Value>
                </Row>
                <Row>
                  <Label>Pool minimum at creation</Label>
                  <Value>{stakeAccount.poolMinimumAtCreation.toNumber()}</Value>
                </Row>
                <Row>
                  <Label>Pending unstake requests</Label>
                  <Value>{stakeAccount.pendingUnstakeRequests}</Value>
                </Row>
                <Row>
                  <Label>Stake amount</Label>
                  <Value>{stakeAccount.stakeAmount.toNumber()}</Value>
                </Row>
              </>
            )}
          </InnerCard>
        </Card>
      </CardContainer>
    </Container>
  );
};

export default InspectPool;
