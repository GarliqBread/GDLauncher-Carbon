import adSize from "@/utils/adhelper";

export const AdsBanner = () => {
  console.log("RERENDERING ADS BANNER");
  return (
    <div
      style={{
        height: `${adSize.height}px`,
        width: `${adSize.width}px`,
      }}
      class="bg-red"
    />
  );
};
