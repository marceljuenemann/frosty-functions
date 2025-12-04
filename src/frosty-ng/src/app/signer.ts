import { Injectable, signal } from '@angular/core';
import { FunctionState } from 'declarations/frosty-functions-backend/frosty-functions-backend.did';
import { Contract, ethers, Network, Transaction, TransactionResponse } from 'ethers';
import bridgeAbi from '../../../../contracts/Bride.abi.json';

@Injectable({
  providedIn: 'root',
})
export class SignerService {

  async invokeFrostyFunction(chainId: number, functionState: FunctionState, calldata: Uint8Array, amount: BigInt): Promise<TransactionResponse> {
    const provider = await this.providerForChain(chainId);
    const signer = await provider.getSigner();
    const address = await signer.getAddress();
    console.log("Invoking function from address: ", address);

    const contract = new Contract(
      this.bridgeAddress(chainId),
      bridgeAbi,
      signer
    );

    const tx = await contract['invokeFunction'](
      functionState.hash,
      calldata,
      {
        value: amount,
        chainId
      }
    );
    console.log('Transaction sent:', tx);
    return tx;
  }

  // TODO: Move into a config file.
  private bridgeAddress(chainId: number): string {
    switch (chainId) {
      case 31337:
        return '0x5FbDB2315678afecb367f032d93F642f64180aa3';
      case 42161:  // Arbitrum One
      case 421614:  // Arbitrum Sepolia
        return '0xe712A7e50abA019A6d225584583b09C4265B037B';
      default:
        throw new Error(`Unsupported chain ID: ${chainId}`);
    }
  }

  async providerForChain(chainId: number): Promise<ethers.BrowserProvider> {
    if ((window as any).ethereum == null) throw new Error("MetaMask not installed");
    const provider =  new ethers.BrowserProvider((window as any).ethereum);
    await provider.send('wallet_switchEthereumChain', [{
      chainId: '0x' + Number(chainId).toString(16)
    }]);

    const network = await provider.getNetwork();
    if (network.chainId !== BigInt(chainId)) {
      throw new Error(`Connected to wrong network: ${network.chainId}, expected ${chainId}`);
    }
    return provider;
  }
}
