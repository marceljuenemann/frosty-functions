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
        return '0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0';
        //return '0x5FbDB2315678afecb367f032d93F642f64180aa3';
      case 42161:  // Arbitrum One
        return '0x8cb969fbba3adc0cc1347116396b3ed1d0bafea1';
      case 421614:  // Arbitrum Sepolia
        return '0xcAcbb4E46F2a68e3d178Fb98dCaCe59d12d54CBc';
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
