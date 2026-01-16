import { Component, signal, Signal } from '@angular/core';
import { ActivatedRoute } from '@angular/router';
import { Chain, Commit, Job } from 'declarations/frosty-functions-backend/frosty-functions-backend.did';
import { FrostyFunctionService } from '../frosty-function-service';
import { encodeHex, formatTimestamp } from '../util';
import { JsonPipe } from '@angular/common';
import { SCANNER_URL } from '../invoke-function/invoke-function';
import { LogViewer } from '../log-viewer/log-viewer';

@Component({
  selector: 'app-job',
  imports: [LogViewer],
  templateUrl: './job.html',
  styleUrl: './job.scss',
})
export class JobComponent {
  job = signal<Job | 'notfound' | 'loading'>('loading');
  commits = signal<Array<Commit>>([]);
  commitCache = new Map<bigint, Promise<Commit>>();

  constructor(private route: ActivatedRoute, private service: FrostyFunctionService) {
    const chain = this.parseChainId(route.snapshot.params['chainId'])
    const jobId = parseInt(route.snapshot.params['jobId']);
    if (!chain || isNaN(jobId)) {
      this.job.set('notfound');
    } else {
      // TODO: Unsubscribe
      this.service.watchJob(chain, jobId).subscribe((job) => {
        this.job.set(job ?? 'notfound');
        this.fetchCommits(job!.commit_ids);
      });
    }
  }

  // TODO: Move somewhere shared.
  private parseChainId(chainId: string): Chain | null {
    switch (chainId) {
      case 'eip155:31337':
        return { Evm: { Localhost: null } };
      case 'eip155:42161':
        return { Evm: { ArbitrumOne: null } };
      case 'eip155:421614':
        return { Evm: { ArbitrumSepolia: null } };
      default:
        return null;
    }
  }

  private async fetchCommits(commitIds: BigUint64Array | bigint[]) {
    console.log("Fetching commits for IDs:", commitIds);
    const promises: Promise<Commit>[] = Array.from(commitIds).map((id) => {
      if (!this.commitCache.has(id)) {
        this.commitCache.set(id, this.service.getCommit(id));
      }
      return this.commitCache.get(id)!;
    });
    console.log("Fetching commits:", promises);
    this.commits.set(await Promise.all(promises));
  }

  chainName(chain: Chain): string {
    if ('Evm' in chain) {
      if ('Localhost' in chain.Evm) return "Localhost EVM Node";
      if ('ArbitrumOne' in chain.Evm) return "Arbitrum One";
      if ('ArbitrumSepolia' in chain.Evm) return "Arbitrum Sepolia Testnet";
    }
    return "Unknown Chain";
  }

  chainId(chain: Chain): string {
    return this.service.chainId(chain).toString();
  }

  jobStatus(job: Job): string {
    // TODO: Move to ic-reactor for better typing (hopefully).
    return Object.keys(job.status)[0].toLowerCase();
  }

  formatTimestamp = formatTimestamp;
  encodeHex = encodeHex;
  SCANNER_URL = SCANNER_URL;
}
