import { Component, signal, Signal } from '@angular/core';
import { ActivatedRoute } from '@angular/router';
import { Chain, Job } from 'declarations/frosty-functions-backend/frosty-functions-backend.did';
import { FrostyFunctionService } from '../frosty-function-service';

@Component({
  selector: 'app-job',
  imports: [],
  templateUrl: './job.html',
  styleUrl: './job.scss',
})
export class JobComponent {
  job = signal<Job | 'notfound' | 'loading'>('loading');

  constructor(private route: ActivatedRoute, private service: FrostyFunctionService) {
    const chain = this.parseChainId(route.snapshot.params['chainId'])
    const jobId = parseInt(route.snapshot.params['jobId']);
    if (!chain || isNaN(jobId)) {
      this.job.set('notfound');
    } else {
      this.service.getJob(chain, jobId).then((job) => {
        this.job.set(job ?? 'notfound');
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
}
