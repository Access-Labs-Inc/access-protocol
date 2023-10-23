import * as BN from 'bn.js'

/**
 * 64-bit value
 */
export class u64 extends BN.BN {
  constructor(
    input: string | number | BN | Uint8Array | Buffer | number[],
    number?: number
  ) {
    super(input, number ?? 10);
  }
  /**
   * Convert to Buffer representation
   */
  toBuffer(
    endian?: BN.Endianness | undefined,
    length?: number | undefined
  ): Buffer {
    const a = super.toArray().reverse();
    const b = Buffer.from(a);
    if (b.length === 8) {
      return b;
    }
    if (b.length > 8) {
      throw new Error("u64 too large");
    }

    const zeroPad = Buffer.alloc(8);
    b.copy(zeroPad);
    return zeroPad;
  }

  /**
   * Construct a u64 from Buffer representation
   */
  static fromBuffer(buffer: Buffer): u64 {
    if (buffer.length !== 8) {
      throw new Error(`Invalid buffer length: ${buffer.length}`);
    }
    return new u64(
      [...buffer]
        .reverse()
        .map((i) => `00${i.toString(16)}`.slice(-2))
        .join(""),
      16
    );
  }
}
