import type {
  JupyterFrontEndPlugin,
  JupyterFrontEnd
} from '@jupyterlab/application';
import { PageConfig, URLExt } from '@jupyterlab/coreutils';

import { IServiceWorkerManager } from '@jupyterlite/apputils';
import type { IKernel } from '@jupyterlite/services';
import { IKernelSpecs } from '@jupyterlite/services';

import { IEmpackEnvMetaFile } from '@jupyterlite/xeus-extension';

import { CxWebWorkerKernel } from './kernel';

interface IKernelListItem {
  env_name: string;
  kernel: string;
}

async function getJson(url: string) {
  const jsonUrl = URLExt.join(PageConfig.getBaseUrl(), url);
  const response = await fetch(jsonUrl, { method: 'GET' });
  if (!response.ok) {
    throw new Error(`HTTP error! status: ${response.status}`);
  }
  return response.json();
}

/**
 * JupyterLite plugin that registers xeus kernels using CxWebWorkerKernel
 * instead of the default WebWorkerKernel. This rewrites %conda commands
 * to %cx so they bypass mambajs and reach real conda in Python.
 *
 * Use alongside disabledExtensions: ["@jupyterlite/xeus-kernel:register"]
 * so the default kernel registration doesn't conflict.
 */
const plugin: JupyterFrontEndPlugin<void> = {
  id: '@conda-express/cx-jupyterlite:register',
  autoStart: true,
  requires: [IKernelSpecs],
  optional: [IServiceWorkerManager, IEmpackEnvMetaFile],
  activate: async (
    app: JupyterFrontEnd,
    kernelspecs: IKernelSpecs,
    serviceWorker?: IServiceWorkerManager,
    empackEnvMetaFile?: IEmpackEnvMetaFile
  ) => {
    let kernelList: IKernelListItem[] = [];
    try {
      kernelList = await getJson('xeus/kernels.json');
    } catch (err) {
      console.log(`cx-jupyterlite: could not fetch xeus/kernels.json: ${err}`);
      throw err;
    }

    const contentsManager = app.serviceManager.contents;
    const kernelNames = kernelList.map(item => item.kernel);
    const duplicateNames = kernelNames.filter(
      (item, index) => kernelNames.indexOf(item) !== index
    );

    for (const kernelItem of kernelList) {
      const { env_name, kernel } = kernelItem;
      const kernelspec = await getJson(
        `xeus/${env_name}/${kernel}/kernel.json`
      );
      kernelspec.name = kernel;
      kernelspec.dir = kernel;
      kernelspec.envName = env_name;

      if (duplicateNames.includes(kernel)) {
        kernelspec.name = `${kernel} (${env_name})`;
        kernelspec.display_name = `${kernelspec.display_name} [${env_name}]`;
      }

      for (const [key, value] of Object.entries(kernelspec.resources)) {
        kernelspec.resources[key] = URLExt.join(
          PageConfig.getBaseUrl(),
          value as string
        );
      }

      kernelspecs.register({
        spec: kernelspec,
        create: async (options: IKernel.IOptions): Promise<IKernel> => {
          const index = kernelspec.name.indexOf(' ');
          if (index > 0) {
            kernelspec.name = kernelspec.name.slice(0, index);
          }

          const mountDrive = !!(serviceWorker?.enabled || crossOriginIsolated);
          const link = empackEnvMetaFile
            ? await empackEnvMetaFile.getLink(kernelspec)
            : '';

          return new CxWebWorkerKernel({
            ...options,
            contentsManager,
            mountDrive,
            kernelSpec: kernelspec,
            empackEnvMetaLink: link,
            browsingContextId: serviceWorker?.browsingContextId || ''
          });
        }
      });
    }

    // @ts-expect-error _specsChanged is internal but needed to trigger UI refresh
    await app.serviceManager.kernelspecs._specsChanged.emit(
      app.serviceManager.kernelspecs.specs
    );

    console.log(
      'cx-jupyterlite: registered kernels with %%conda → %%cx rewrite'
    );
  }
};

export default plugin;
