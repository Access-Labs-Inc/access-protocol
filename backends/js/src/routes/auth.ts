import { Router } from "express";
import { ApiResponse } from "../types/apiResponse";
import { genrateNonce } from "../utils/nonce";
import { ErrorMessage } from "../types/errors";
import { validaRequestBody } from "../utils/validateRequest";
import {
  NonceRequestSchema,
  NonceRequest,
  LoginRequestSchema,
  LoginRequest,
} from "../types/routes";

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

router.post("/login", validaRequestBody(LoginRequestSchema), (req, res) => {
  try {
    const { address, nonce } = req.body as LoginRequest;
    // Check nonce signature
    // Check amount staked
    // JWT
    return res.sendStatus(200);
  } catch (err) {}
});

export default router;
