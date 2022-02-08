import React from "react";
import { styled } from "@mui/material/styles";

const Root = styled("div")({
  boxShadow: "0 4px 8px 0 rgba(0,0,0,0.2)",
  transition: "0.3s",
  "&:hover": {
    boxShadow: "0 8px 16px 0 rgba(0,0,0,0.2)",
  },
  height: "100%",
  width: "100%",
  display: "flex",
  justifyContent: "space-around",
  alignItems: "center",
  flexDirection: "column",
});

const Card = ({ children }: { children: React.ReactNode }) => {
  return <Root>{children}</Root>;
};

export default Card;
