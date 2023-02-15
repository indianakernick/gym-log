import { useModal } from 'vue-final-modal';
import ConfirmModal from '@/modals/ConfirmModal.vue';

type Options = InstanceType<typeof ConfirmModal>['$props'];

export function useConfirmModal(): (options: Options) => Promise<boolean>;
export function useConfirmModal(options: Options): () => Promise<boolean>;
export function useConfirmModal<O extends Partial<Options>>(
  options: O
): (options: Omit<Options, keyof O> & Partial<O>) => Promise<boolean>;
export function useConfirmModal(options?: Partial<Options>) {
  const modal = useModal({
    component: ConfirmModal,
    ...((options ? { attrs: options } : undefined) as any)
  });

  return (options?: Partial<Options>) => {
    return new Promise<boolean>(resolve => {
      modal.patchOptions({
        attrs: {
          ...options,
          onConfirm: (confirmed: boolean) => {
            modal.close();
            resolve(confirmed);
          },
          onClickOutside: () => resolve(false)
        }
      });
      modal.open();
    });
  };
}
