import { Injectable } from '@angular/core';
import { FunctionState } from 'declarations/frosty-functions-backend/frosty-functions-backend.did';

import { ethers } from 'ethers';

@Injectable({
  providedIn: 'root',
})
export class SignerService {
  private provider: ethers.BrowserProvider | null = null;
  private signer: ethers.Signer | null = null;

  async runFrostyFunction(functionState: FunctionState) {
    // TODO: Implement the function invocation logic
    this.getProvider();

  }

  async getProvider(): Promise<ethers.BrowserProvider> {
    if (this.provider) return this.provider;
    if ((window as any).ethereum == null) throw new Error("MetaMask not installed");
    return new ethers.BrowserProvider((window as any).ethereum);
  }

  async getSigner(): Promise<ethers.Signer> {
    if (this.signer) return this.signer;
    const provider = await this.getProvider();
    return provider.getSigner();
  }
}
