import { Router } from "express";
import { ApiResponse } from "../types/apiResponse";
import { genrateNonce, verifyNonce } from "../utils/nonce";
import { ErrorMessage } from "../types/errors";
import { validaRequestBody } from "../utils/validateRequest";
import {
  NonceRequestSchema,
  NonceRequest,
  LoginRequestSchema,
  LoginRequest,
} from "../types/routes";
import jwt from "jsonwebtoken";
import { ACCESS_TOKEN_SECRET } from "../utils/jwt";
import { getNonce, setNonce } from "../utils/redis";
import { checkStake } from "../utils/stake";

const router = Router();

router.post(
  "/nonce",
  validaRequestBody(NonceRequestSchema),
  async (req, res) => {
    try {
      // Generate nonce
      const nonce = genrateNonce();

      // Store nonce in db
      const { address } = req.body as NonceRequest;

      console.log(`Address ${address} - nonce ${nonce}`);

      setNonce(nonce, address);

      return res.json(new ApiResponse(true, { nonce }));
    } catch (err) {
      console.error(`Error generating nonce ${err}`);
      res.status(500);
      return res.json(
        new ApiResponse(false, ErrorMessage.ErrorGeneratingNonce)
      );
    }
  }
);

router.post(
  "/login",
  validaRequestBody(LoginRequestSchema),
  async (req, res) => {
    try {
      const { address, signedNonce } = req.body as LoginRequest;
      const nonce = await getNonce(address);
      console.log(`Stored nonce ${nonce}`);

      if (!nonce) {
        return res.sendStatus(400);
      }

      // Check nonce signature
      const isValidNonce = verifyNonce(nonce, signedNonce, address);
      if (!isValidNonce) {
        return res
          .status(401)
          .json(new ApiResponse(false, ErrorMessage.InvalidNonce));
      }

      // Check amount staked
      const isValidStake = await checkStake(address);
      if (!isValidStake) {
        return res
          .status(401)
          .json(new ApiResponse(false, ErrorMessage.InvalidStake));
      }

      // JWT
      const token = jwt.sign(
        { address, iat: new Date().getTime() },
        ACCESS_TOKEN_SECRET
      );
      return res.json(new ApiResponse(true, { token }));
    } catch (err) {
      console.error(`Error validating nonce ${err}`);
      res.status(500);
      return res.json(
        new ApiResponse(false, ErrorMessage.ErrorValidatingNonce)
      );
    }
  }
);

export default router;
