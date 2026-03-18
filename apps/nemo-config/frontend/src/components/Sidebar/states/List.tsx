import { useSnapshot } from 'valtio';
import { store } from '../../../store';
import { ServiceItem } from '../ServiceItem';

export const SidebarList = () => {
  const { templates } = useSnapshot(store);
  const sortedTemplates = [...templates].sort((a, b) => a.name.localeCompare(b.name));
  
  return (
    <>
      {sortedTemplates.map(template => (
        <ServiceItem key={template.id} templateId={template.id} />
      ))}
    </>
  );
};
