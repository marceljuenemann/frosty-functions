import { Injectable, signal } from '@angular/core';
import { FunctionState } from 'declarations/frosty-functions-backend/frosty-functions-backend.did';
import { Contract, ethers, Network } from 'ethers';
import bridgeAbi from '../../../../contracts/Bride.abi.json';

@Injectable({
  providedIn: 'root',
})
export class SignerService {

  // TODO: Pass calldata
  // TODO: Set contract address (in config somewhere)
  // TODO: Set chain ID

   // TODO: configure
  async runFrostyFunction(functionState: FunctionState) {
    const chainId = 421614;


    const provider = await this.providerForChain(chainId);
    const signer = await provider.getSigner();
    const address = await signer.getAddress();

    console.log('Address:', address);

    const contract = new Contract(
      '0xe712A7e50abA019A6d225584583b09C4265B037B', // Arb Sepolia
      bridgeAbi,
      signer
    );

    const tx = await contract['invokeFunction'](
      functionState.hash,
      Uint8Array.from([]),  // TODO: calldata
      {
        value: 1234,
        chainId
      }
    );
    console.log('Transaction sent:', tx);

    const receipt = await tx.wait();
    console.log('Receipt:', receipt);
    return receipt.hash;
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
