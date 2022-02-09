import React from "react";
import { useParams } from "react-router-dom";
import BondCard from "../components/BondCard";
import { styled } from "@mui/material/styles";

const Center = styled("div")({
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
});

const BondPage = () => {
  const { key } = useParams();
  if (!key) {
    return <span>No public key</span>;
  }
  return (
    <Center>
      <BondCard bondKey={key} />
    </Center>
  );
};

export default BondPage;
