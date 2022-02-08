import React, { useState } from "react";
import Card from "../components/Card";
import { styled } from "@mui/material/styles";
import { useWallet, useConnection } from "@solana/wallet-adapter-react";
import CircularProgress from "@mui/material/CircularProgress";
import FormControl from "@mui/material/FormControl";
import InputLabel from "@mui/material/InputLabel";
import OutlinedInput from "@mui/material/OutlinedInput";
import { Button } from "@mui/material";
import { sendTx } from "../utils/send";
import { changeInflation, ACCESS_PROGRAM_ID } from "@access";
import { notify } from "../utils/notifications";
import { refreshAllCaches } from "../utils/fetch-loop";

const CardContainer = styled("div")({
  height: 400,
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  flexDirection: "column",
});

const Title = styled("span")({
  fontSize: 25,
});

const FormControlStyled = styled(FormControl)({
  width: "90%",
});

const EditInflation = () => {
  const [loading, setLoading] = useState(false);
  const [inflation, setInflation] = useState<null | number>(null);
  const { connected, publicKey, sendTransaction } = useWallet();
  const { connection } = useConnection();
  const handle = async () => {
    if (!inflation || !publicKey) return;
    try {
      setLoading(true);
      const ix = await changeInflation(
        connection,
        inflation,
        ACCESS_PROGRAM_ID
      );
      const tx = await sendTx(connection, publicKey, [ix], sendTransaction);
      notify({ message: "Inflation changed", variant: "success" });
      console.log(tx);
    } catch (err) {
      console.log(err);
      notify({ message: `Error ${err}`, variant: "error" });
    } finally {
      setLoading(false);
      refreshAllCaches();
    }
  };
  return (
    <CardContainer>
      <Card>
        <Title>Edit protocol inflation</Title>
        <FormControlStyled>
          <InputLabel>New inflation</InputLabel>
          <OutlinedInput
            type="number"
            id="component-outlined"
            value={inflation}
            onChange={(e) => setInflation(parseInt(e.target.value.trim()))}
            label="New inflation"
          />
        </FormControlStyled>
        <Button disabled={!connected} variant="contained" onClick={handle}>
          {loading ? <CircularProgress color="inherit" /> : "Edit"}
        </Button>
      </Card>
    </CardContainer>
  );
};

export default EditInflation;
